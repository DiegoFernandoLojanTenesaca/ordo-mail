use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, RwLock};
use std::{thread, time::Duration};

use serde_json::{Value, json};

use crate::auth::{self, Client, LoginPage};
use crate::config::{BATCH_SIZE, COUNT_CAP, GMAIL_API, MAX_BACKOFF_SECS, MAX_RETRIES, PAGE_SIZE, PROGRESS_EVERY, QUERY_SPAM, THREADS};
use crate::error::{Error, ErrorCode, Result};
use crate::model::{Phase, Progress, Report};
use crate::settings::Storage;
use crate::text::{email_of, encode, name_of};

pub(crate) struct Message {
    pub from: String,
    pub name: String,
    pub subject: String,
}

pub struct Gmail {
    agent: ureq::Agent,
    client: Client,
    refresh_token: String,
    access_token: RwLock<String>,
    pub account: String,
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder().http_status_as_error(false).build().into()
}

impl Gmail {
    pub fn restore(storage: &Storage) -> Result<Option<Self>> {
        let Some(refresh_token) = storage.load_token()? else { return Ok(None) };
        let (agent, client) = (agent(), Client::load(storage)?);
        match auth::refresh(&agent, &client, &refresh_token) {
            Ok(access) => Self::open(agent, client, refresh_token, access).map(Some),
            Err(e) if e.code == ErrorCode::TokenRejected => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn login(storage: &Storage, page: &LoginPage) -> Result<Self> {
        let (agent, client) = (agent(), Client::load(storage)?);
        let (access, refresh_token) = auth::login(&agent, &client, page)?;
        storage.save_token(&refresh_token)?;
        Self::open(agent, client, refresh_token, access)
    }

    fn open(agent: ureq::Agent, client: Client, refresh_token: String, access: String) -> Result<Self> {
        let mut gmail = Self { agent, client, refresh_token, access_token: RwLock::new(access), account: String::new() };
        gmail.account = gmail.get("/profile")?["emailAddress"].as_str().unwrap_or_default().to_lowercase();
        Ok(gmail)
    }

    fn request(&self, method: &str, path: &str, body: Option<&Value>) -> Result<Value> {
        let url = format!("{GMAIL_API}{path}");
        for attempt in 0..MAX_RETRIES {
            let bearer = format!("Bearer {}", self.access_token.read().unwrap());
            let mut response = match (method, body) {
                ("DELETE", _) => self.agent.delete(&url).header("Authorization", &bearer).call()?,
                ("PATCH", Some(b)) => self.agent.patch(&url).header("Authorization", &bearer).send_json(b)?,
                (_, Some(b)) => self.agent.post(&url).header("Authorization", &bearer).send_json(b)?,
                _ => self.agent.get(&url).header("Authorization", &bearer).call()?,
            };
            let status = response.status().as_u16();
            let text = response.body_mut().read_to_string()?;
            if status == 401 && attempt == 0 {
                *self.access_token.write().unwrap() = auth::refresh(&self.agent, &self.client, &self.refresh_token)?;
                continue;
            }
            if status == 429 || status >= 500 || (status == 403 && text.to_lowercase().contains("exceeded")) {
                thread::sleep(Duration::from_secs((1 << attempt).min(MAX_BACKOFF_SECS)));
                continue;
            }
            if status >= 400 {
                return Err(Error::with(ErrorCode::Gmail, format!("{status} {text}")));
            }
            return Ok(if text.is_empty() { Value::Null } else { serde_json::from_str(&text)? });
        }
        Err(ErrorCode::GmailBusy.into())
    }

    pub fn get(&self, path: &str) -> Result<Value> {
        self.request("GET", path, None)
    }

    pub fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.request("POST", path, Some(body))
    }

    pub fn patch(&self, path: &str, body: &Value) -> Result<Value> {
        self.request("PATCH", path, Some(body))
    }

    pub fn delete(&self, path: &str) -> Result<Value> {
        self.request("DELETE", path, None)
    }

    pub fn user_labels(&self) -> Result<Vec<(String, String)>> {
        Ok(self.get("/labels")?["labels"].as_array().into_iter().flatten()
            .filter(|l| l["type"] == "user")
            .map(|l| (l["id"].as_str().unwrap_or_default().to_string(), l["name"].as_str().unwrap_or_default().to_string()))
            .collect())
    }

    pub fn create_label(&self, name: &str) -> Result<String> {
        let label = self.post("/labels", &json!({ "name": name, "labelListVisibility": "labelShow", "messageListVisibility": "show" }))?;
        Ok(label["id"].as_str().unwrap_or_default().to_string())
    }

    pub fn filters(&self) -> Result<Vec<Value>> {
        Ok(self.get("/settings/filters")?["filter"].as_array().cloned().unwrap_or_default())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        let mut page: Option<String> = None;
        while ids.len() < limit {
            let mut path = format!("/messages?maxResults={PAGE_SIZE}&q={}", encode(query));
            if query.contains(QUERY_SPAM) {
                path += "&includeSpamTrash=true";
            }
            if let Some(p) = &page {
                path += &format!("&pageToken={}", encode(p));
            }
            let response = self.get(&path)?;
            ids.extend(response["messages"].as_array().into_iter().flatten().filter_map(|m| m["id"].as_str().map(String::from)));
            page = response["nextPageToken"].as_str().map(String::from);
            if page.is_none() {
                break;
            }
        }
        ids.truncate(limit);
        Ok(ids)
    }

    pub fn count(&self, query: &str) -> Result<(usize, bool)> {
        let n = self.search(query, COUNT_CAP)?.len();
        Ok((n, n < COUNT_CAP))
    }

    pub fn modify(&self, ids: &[String], add: &[&str], remove: &[&str]) -> Result<()> {
        for batch in ids.chunks(BATCH_SIZE) {
            self.post("/messages/batchModify", &json!({ "ids": batch, "addLabelIds": add, "removeLabelIds": remove }))?;
        }
        Ok(())
    }

    pub(crate) fn read(&self, query: &str, limit: usize, report: Report) -> Result<(Vec<Message>, usize)> {
        report(Progress::new(Phase::Searching, 0, 0));
        let ids = self.search(query, limit)?;
        let done = AtomicUsize::new(0);
        let messages = parallel(&ids, |id| {
            let message = self.get(&format!("/messages/{id}?format=metadata&metadataHeaders=From&metadataHeaders=Subject"));
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if n.is_multiple_of(PROGRESS_EVERY) || n == ids.len() {
                report(Progress::new(Phase::Reading, n, ids.len()));
            }
            message.ok().map(|m| {
                let header = |name: &str| m["payload"]["headers"].as_array().into_iter().flatten()
                    .find(|h| h["name"] == name).and_then(|h| h["value"].as_str()).unwrap_or_default().to_string();
                let from = header("From");
                Message { from: email_of(&from), name: name_of(&from), subject: header("Subject") }
            })
        });
        let failed = messages.iter().filter(|m| m.is_none()).count();
        Ok((messages.into_iter().flatten().collect(), failed))
    }
}

pub fn parallel<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let out = Mutex::new(Vec::with_capacity(items.len()));
    thread::scope(|s| {
        for _ in 0..THREADS.min(items.len()) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some(item) = items.get(i) else { break };
                let r = f(item);
                out.lock().unwrap().push((i, r));
            });
        }
    });
    let mut results = out.into_inner().unwrap();
    results.sort_by_key(|(i, _)| *i);
    results.into_iter().map(|(_, r)| r).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn parallel_keeps_order() {
        let v: Vec<usize> = (0..100).collect();
        assert_eq!(super::parallel(&v, |x| x * 2), (0..100).map(|x| x * 2).collect::<Vec<_>>());
    }
}
