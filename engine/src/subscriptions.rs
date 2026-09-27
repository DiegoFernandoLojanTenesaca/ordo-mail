use std::cmp::Reverse;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::{
    MIN_MESSAGES_PER_SENDER, ONE_CLICK_KEY, ONE_CLICK_VALUE, QUERY_ALL, QUERY_ANYWHERE, UNSUBSCRIBE_SAMPLES, UNSUBSCRIBE_TIMEOUT_SECS,
};
use crate::error::{Error, ErrorCode, Result};
use crate::gmail::{Gmail, Message, header};
use crate::model::{Phase, Progress, Report, Subscription, Unsubscribe};
use crate::settings::Settings;
use crate::text::{unsubscribe_kind, unsubscribe_targets, valid_sender};

pub fn scan(gmail: &Gmail, settings: &Settings, report: Report) -> Result<Vec<Subscription>> {
    let (messages, _) = gmail.read(QUERY_ALL, settings.limit, report)?;
    let found = summarize(&messages, &gmail.account);
    report(Progress::new(Phase::Done, found.len(), found.len()));
    Ok(found)
}

fn summarize(messages: &[Message], account: &str) -> Vec<Subscription> {
    let mut by_sender: HashMap<&str, Subscription> = HashMap::new();
    for m in messages {
        let s = by_sender.entry(&m.from).or_insert_with(|| Subscription {
            email: m.from.clone(),
            name: String::new(),
            count: 0,
            unread: 0,
            unsubscribe: Unsubscribe::None,
        });
        s.count += 1;
        s.unread += usize::from(m.unread);
        s.unsubscribe = s.unsubscribe.max(m.unsubscribe);
        if s.name.is_empty() {
            s.name.clone_from(&m.name);
        }
    }
    let mut found: Vec<Subscription> = by_sender
        .into_values()
        .filter(|s| s.count >= MIN_MESSAGES_PER_SENDER && s.unsubscribe != Unsubscribe::None && s.email != account)
        .collect();
    found.sort_by_key(|s| (Reverse(s.count), s.email.clone()));
    found
}

pub fn unsubscribe(gmail: &Gmail, email: &str) -> Result<Unsubscribe> {
    if !email.contains('@') || !valid_sender(email) {
        return Err(Error::with(ErrorCode::InvalidSender, email));
    }
    let (list, post) = gmail
        .search(&format!("from:{email} {QUERY_ANYWHERE}"), UNSUBSCRIBE_SAMPLES)?
        .iter()
        .filter_map(|id| gmail.metadata(id).ok())
        .map(|m| (header(&m, "List-Unsubscribe"), header(&m, "List-Unsubscribe-Post")))
        .max_by_key(|(list, post)| unsubscribe_kind(list, post))
        .unwrap_or_default();
    let (web, mail) = unsubscribe_targets(&list);
    let kind = unsubscribe_kind(&list, &post);
    match (kind, web, mail) {
        (Unsubscribe::OneClick, Some(web), _) if one_click(&web) => Ok(Unsubscribe::OneClick),
        (Unsubscribe::OneClick | Unsubscribe::Link, Some(web), _) => launch(&web, Unsubscribe::Link),
        (Unsubscribe::Mail, _, Some(mail)) => launch(&mail, Unsubscribe::Mail),
        _ => Err(ErrorCode::NoUnsubscribe.into()),
    }
}

fn one_click(url: &str) -> bool {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .timeout_global(Some(Duration::from_secs(UNSUBSCRIBE_TIMEOUT_SECS)))
        .build()
        .into();
    agent
        .post(url)
        .send_form([(ONE_CLICK_KEY, ONE_CLICK_VALUE)])
        .is_ok_and(|r| r.status().as_u16() < 400)
}

fn launch(target: &str, kind: Unsubscribe) -> Result<Unsubscribe> {
    open::that_detached(target).map_err(|e| Error::with(ErrorCode::UnsubscribeFailed, e))?;
    Ok(kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(from: &str, unread: bool, unsubscribe: Unsubscribe) -> Message {
        Message {
            from: from.into(),
            name: "News".into(),
            subject: String::new(),
            unread,
            unsubscribe,
        }
    }

    #[test]
    fn subscriptions_are_grouped_by_sender() {
        let messages = [
            message("news@shop.example", true, Unsubscribe::Link),
            message("news@shop.example", true, Unsubscribe::OneClick),
            message("news@shop.example", false, Unsubscribe::None),
            message("deals@store.example", true, Unsubscribe::Mail),
            message("deals@store.example", true, Unsubscribe::Mail),
            message("friend@mail.example", false, Unsubscribe::None),
            message("friend@mail.example", false, Unsubscribe::None),
            message("once@list.example", true, Unsubscribe::Link),
            message("me@mail.example", true, Unsubscribe::Link),
            message("me@mail.example", true, Unsubscribe::Link),
        ];
        let found = summarize(&messages, "me@mail.example");
        assert_eq!(found.len(), 2);
        assert_eq!(
            (found[0].email.as_str(), found[0].count, found[0].unread),
            ("news@shop.example", 3, 2)
        );
        assert_eq!(found[0].unsubscribe, Unsubscribe::OneClick);
        assert_eq!((found[1].count, found[1].unread, found[1].unsubscribe), (2, 2, Unsubscribe::Mail));
    }
}
