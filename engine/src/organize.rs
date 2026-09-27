use std::cmp::Reverse;
use std::collections::HashMap;

use crate::claude::{self, Sample};
use crate::config::{MAX_SENDERS_FOR_CLAUDE, MIN_MESSAGES_PER_SENDER, QUERY_ALL, QUERY_SPAM, QUERY_UNLABELED, SUBJECTS_PER_SENDER};
use crate::error::Result;
use crate::gmail::Gmail;
use crate::model::{Action, Group, Phase, Progress, Proposal, Report, Sender, Summary};
use crate::rules::{destination_of, destinations};
use crate::settings::Settings;

pub fn summary(gmail: &Gmail) -> Result<Summary> {
    let (unlabeled, unlabeled_complete) = gmail.count(QUERY_UNLABELED)?;
    Ok(Summary {
        unlabeled,
        unlabeled_complete,
        labels: gmail.user_labels()?.len(),
        rules: gmail.filters()?.len(),
        spam: gmail.count(QUERY_SPAM)?.0,
    })
}

pub fn analyze(gmail: &Gmail, settings: &Settings, reorganize: bool, locale: &str, report: Report) -> Result<Proposal> {
    let query = if reorganize { QUERY_ALL } else { QUERY_UNLABELED };
    let (messages, failed) = gmail.read(query, settings.limit, report)?;
    let own = gmail.user_labels()?;
    let names: HashMap<&str, &str> = own.iter().map(|(id, name)| (id.as_str(), name.as_str())).collect();
    let destinations = destinations(&gmail.filters()?);
    let current = |email: &str| destination_of(&destinations, email).and_then(|id| names.get(id).copied());

    let mut by_sender: HashMap<&str, Sample> = HashMap::new();
    for m in &messages {
        let sample = by_sender.entry(&m.from).or_insert_with(|| Sample {
            email: &m.from, name: &m.name, count: 0, subjects: Vec::new(), current: current(&m.from),
        });
        sample.count += 1;
        if sample.subjects.len() < SUBJECTS_PER_SENDER && !m.subject.is_empty() {
            sample.subjects.push(&m.subject);
        }
    }
    let mut samples: Vec<Sample> = by_sender.into_values()
        .filter(|s| s.count >= MIN_MESSAGES_PER_SENDER && s.email != gmail.account)
        .collect();
    samples.sort_by_key(|s| Reverse(s.count));
    samples.truncate(MAX_SENDERS_FOR_CLAUDE);
    let read = messages.len();
    if samples.is_empty() {
        return Ok(Proposal { read, failed, groups: Vec::new() });
    }

    report(Progress::about(Phase::Thinking, samples.len().to_string(), 0, 0));
    let existing: Vec<String> = own.iter().map(|(_, name)| name.clone()).collect();
    let count_of = |sender: &str| -> usize {
        samples.iter().filter(|s| s.email == sender || s.email.ends_with(&format!("@{sender}"))).map(|s| s.count).sum()
    };
    let mut groups: Vec<Group> = claude::classify(settings.model, locale, &samples, &existing)?
        .into_iter()
        .map(|(label, senders)| Group {
            is_new: !existing.iter().any(|e| e.eq_ignore_ascii_case(&label)),
            action: Action::default(),
            senders: senders.into_iter()
                .map(|email| Sender { count: count_of(&email), current: current(&email).map(String::from), email })
                .filter(|s| !s.current.as_deref().is_some_and(|c| c.eq_ignore_ascii_case(&label)))
                .collect(),
            label,
        })
        .filter(|g| !g.senders.is_empty())
        .collect();
    groups.sort_by_key(|g| Reverse(g.senders.len()));
    Ok(Proposal { read, failed, groups })
}
