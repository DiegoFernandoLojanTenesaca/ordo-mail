use std::cmp::Reverse;
use std::collections::HashMap;

use serde_json::{Value, json};

use crate::config::LABEL_LENGTH;
use crate::error::{Error, ErrorCode, Result};
use crate::gmail::{Gmail, parallel};
use crate::model::{Action, Group, Label, Phase, Progress, Report, Rule};
use crate::settings::Settings;
use crate::text::valid_sender;

impl Action {
    fn changes(self, label: &str) -> (Vec<&str>, Vec<&str>) {
        match self {
            Action::Label => (vec![label], vec![]),
            Action::Archive => (vec![label], vec!["INBOX"]),
            Action::Trash => (vec![label, "TRASH"], vec![]),
        }
    }

    fn of_filter(filter: &Value) -> Self {
        let has = |field: &str, id: &str| filter["action"][field].as_array().is_some_and(|a| a.iter().any(|x| x == id));
        if has("addLabelIds", "TRASH") {
            Action::Trash
        } else if has("removeLabelIds", "INBOX") {
            Action::Archive
        } else {
            Action::Label
        }
    }
}

fn filter(sender: &str, label: &str, action: Action) -> Value {
    let (add, remove) = action.changes(label);
    let mut f = json!({ "criteria": { "from": sender }, "action": { "addLabelIds": add } });
    if !remove.is_empty() {
        f["action"]["removeLabelIds"] = json!(remove);
    }
    f
}

fn adds(filter: &Value, label: &str) -> bool {
    filter["action"]["addLabelIds"].as_array().is_some_and(|a| a.iter().any(|x| x == label))
}

fn is_simple(filter: &Value) -> bool {
    filter["criteria"].as_object().is_some_and(|c| c.len() == 1 && c.contains_key("from"))
}

fn user_labels_of(filter: &Value) -> impl Iterator<Item = &str> {
    filter["action"]["addLabelIds"].as_array().into_iter().flatten().filter_map(Value::as_str).filter(|l| l.starts_with("Label_"))
}

fn checked_id(id: &str) -> Result<&str> {
    if !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || "-_".contains(c)) { Ok(id) } else { Err(ErrorCode::InvalidId.into()) }
}

fn checked_name(name: &str) -> Result<&str> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > LABEL_LENGTH || name.contains(['"', '\n']) {
        return Err(Error::with(ErrorCode::InvalidLabel, name));
    }
    Ok(name)
}

pub(crate) fn destinations(filters: &[Value]) -> Vec<(String, String)> {
    filters.iter().filter(|f| is_simple(f))
        .filter_map(|f| Some((f["criteria"]["from"].as_str()?.to_lowercase(), user_labels_of(f).next()?.to_string())))
        .collect()
}

pub(crate) fn destination_of<'a>(destinations: &'a [(String, String)], email: &str) -> Option<&'a str> {
    destinations.iter().find(|(from, _)| from == email)
        .or_else(|| destinations.iter().find(|(from, _)| email.ends_with(&format!("@{from}"))))
        .map(|(_, id)| id.as_str())
}

pub fn labels(gmail: &Gmail, settings: &Settings) -> Result<Vec<Label>> {
    let (own, filters) = (gmail.user_labels()?, gmail.filters()?);
    let totals = parallel(&own, |(id, _)| gmail.get(&format!("/labels/{id}")).map_or(0, |l| l["messagesTotal"].as_u64().unwrap_or(0) as u32));
    let mut labels: Vec<Label> = own.into_iter().zip(totals)
        .map(|((id, name), total)| Label {
            rules: filters.iter().filter(|f| adds(f, &id)).map(|f| Rule {
                filter_id: f["id"].as_str().unwrap_or_default().into(),
                sender: f["criteria"]["from"].as_str().unwrap_or_default().into(),
                action: Action::of_filter(f),
            }).collect(),
            protected: settings.is_protected(&name),
            id,
            name,
            total,
        })
        .collect();
    labels.sort_by_key(|l| Reverse(l.total));
    Ok(labels)
}

pub fn apply(gmail: &Gmail, groups: &[Group], report: Report) -> Result<()> {
    let mut ids: HashMap<String, String> = gmail.user_labels()?.into_iter().map(|(id, name)| (name.to_lowercase(), id)).collect();
    let filters = gmail.filters()?;
    let destinations = destinations(&filters);
    for group in groups {
        checked_name(&group.label)?;
        if let Some(bad) = group.senders.iter().find(|s| !valid_sender(&s.email)) {
            return Err(Error::with(ErrorCode::InvalidSender, &bad.email));
        }
    }
    let total: usize = groups.iter().map(|g| g.senders.len()).sum();
    let mut done = 0;
    for group in groups {
        let name = checked_name(&group.label)?;
        let id = match ids.get(&name.to_lowercase()) {
            Some(id) => id.clone(),
            None => {
                let id = gmail.create_label(name)?;
                ids.insert(name.to_lowercase(), id.clone());
                id
            }
        };
        for sender in &group.senders {
            report(Progress::about(Phase::Applying, format!("{} → {name}", sender.email), done, total));
            let wanted = filter(&sender.email, &id, group.action);
            let mut previous: Vec<String> = destination_of(&destinations, &sender.email).filter(|old| *old != id).map(String::from).into_iter().collect();
            for f in filters.iter().filter(|f| is_simple(f) && f["criteria"]["from"] == sender.email.as_str() && !adds(f, &id)) {
                gmail.delete(&format!("/settings/filters/{}", checked_id(f["id"].as_str().unwrap_or_default())?))?;
                previous.extend(user_labels_of(f).map(String::from));
            }
            if !filters.iter().any(|f| f["criteria"] == wanted["criteria"] && f["action"] == wanted["action"]) {
                gmail.post("/settings/filters", &wanted)?;
            }
            let existing = if group.action == Action::Trash { Action::Label } else { group.action };
            let (add, mut remove) = existing.changes(&id);
            remove.extend(previous.iter().map(String::as_str));
            gmail.modify(&gmail.search(&format!("from:({})", sender.email), usize::MAX)?, &add, &remove)?;
            done += 1;
        }
    }
    report(Progress::new(Phase::Done, total, total));
    Ok(())
}

pub fn change_action(gmail: &Gmail, label_id: &str, action: Action) -> Result<()> {
    for f in gmail.filters()?.iter().filter(|f| adds(f, label_id) && is_simple(f) && Action::of_filter(f) != action) {
        gmail.delete(&format!("/settings/filters/{}", checked_id(f["id"].as_str().unwrap_or_default())?))?;
        gmail.post("/settings/filters", &filter(f["criteria"]["from"].as_str().unwrap_or_default(), label_id, action))?;
    }
    Ok(())
}

pub fn rename_label(gmail: &Gmail, label_id: &str, name: &str) -> Result<String> {
    let id = checked_id(label_id)?;
    let old = gmail.user_labels()?.into_iter().find(|(l, _)| l == id).map(|(_, n)| n).ok_or(ErrorCode::InvalidId)?;
    gmail.patch(&format!("/labels/{id}"), &json!({ "name": checked_name(name)? }))?;
    Ok(old)
}

pub fn delete_label(gmail: &Gmail, label_id: &str) -> Result<()> {
    let id = checked_id(label_id)?;
    for f in gmail.filters()?.iter().filter(|f| adds(f, id)) {
        gmail.delete(&format!("/settings/filters/{}", checked_id(f["id"].as_str().unwrap_or_default())?))?;
    }
    gmail.delete(&format!("/labels/{id}"))?;
    Ok(())
}

pub fn remove_rule(gmail: &Gmail, filter_id: &str) -> Result<()> {
    gmail.delete(&format!("/settings/filters/{}", checked_id(filter_id)?))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_reads_back_as_written() {
        for action in [Action::Label, Action::Archive, Action::Trash] {
            let f = filter("a.com", "Label_1", action);
            assert_eq!(Action::of_filter(&f), action);
            assert!(adds(&f, "Label_1") && is_simple(&f));
        }
    }

    #[test]
    fn exact_rules_win_over_domain_rules() {
        let d = destinations(&[filter("linkedin.com", "Label_1", Action::Label), filter("jobs@linkedin.com", "Label_2", Action::Archive)]);
        assert_eq!(destination_of(&d, "jobs@linkedin.com"), Some("Label_2"));
        assert_eq!(destination_of(&d, "news@linkedin.com"), Some("Label_1"));
        assert_eq!(destination_of(&d, "other@x.com"), None);
    }

    #[test]
    fn ids_and_names_are_checked() {
        assert!(checked_id("ANe1Bmj").is_ok() && checked_id("../x").is_err());
        assert_eq!(checked_name("  Work/Acme ").unwrap(), "Work/Acme");
        assert!(checked_name("").is_err() && checked_name(&"x".repeat(41)).is_err() && checked_name("a\"b").is_err());
    }
}
