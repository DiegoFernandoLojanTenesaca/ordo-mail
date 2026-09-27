use std::collections::{BTreeMap, HashSet};

use serde_json::Value;

use crate::config::{LABEL_LENGTH, PUBLIC_DOMAINS, SUBJECT_LENGTH};
use crate::error::{ErrorCode, Result};
use crate::model::{Phase, Progress, Report};
use crate::settings::valid_locale;
use crate::text::valid_sender;

pub(crate) struct Sample<'a> {
    pub email: &'a str,
    pub name: &'a str,
    pub count: usize,
    pub subjects: Vec<&'a str>,
    pub current: Option<&'a str>,
}

pub(crate) fn classify(
    ask: impl Fn(&str) -> Result<Value>,
    batch: usize,
    locale: &str,
    samples: &[Sample],
    existing: &[String],
    report: Report,
) -> Result<BTreeMap<String, Vec<String>>> {
    if !valid_locale(locale) {
        return Err(ErrorCode::InvalidLocale.into());
    }
    let batches: Vec<&[Sample]> = samples.chunks(batch.max(1)).collect();
    let total = if batches.len() > 1 { batches.len() } else { 0 };
    let mut labels = existing.to_vec();
    let mut seen = HashSet::new();
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (i, chunk) in batches.iter().enumerate() {
        report(Progress::about(Phase::Thinking, samples.len().to_string(), i, total));
        let answer = ask(&prompt(locale, chunk, &labels))?;
        let known: HashSet<&str> = chunk.iter().map(|s| s.email).collect();
        for rule in answer["rules"].as_array().into_iter().flatten() {
            let label = rule["label"].as_str().unwrap_or_default().trim();
            if label.is_empty() || label.chars().count() > LABEL_LENGTH || label.contains(['=', '#', '"', '\n']) {
                continue;
            }
            let label = match labels.iter().find(|e| e.eq_ignore_ascii_case(label)) {
                Some(known_label) => known_label.clone(),
                None => {
                    labels.push(label.to_string());
                    label.to_string()
                }
            };
            for sender in rule["senders"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                let sender = sender.trim().to_lowercase();
                if acceptable(&sender, &known) && seen.insert(sender.clone()) {
                    out.entry(label.clone()).or_default().push(sender);
                }
            }
        }
    }
    Ok(out)
}

fn prompt(locale: &str, samples: &[Sample], labels: &[String]) -> String {
    let mut prompt = format!(
        "You organize a Gmail inbox. Below are its senders in the format:\n\
         message count | name <email> | current label (- if none) | sample subjects.\n\n\
         Give every sender the most specific useful label:\n\
         - Every company, client, project or work contact the person deals with gets its own label (e.g. \"Work/Acme\"), never a generic bucket.\n\
         - Services and paperwork are separate too: invoices, the internet provider (\"Services/Internet\"), each bank on its own, insurance, government, university.\n\
         - Generic buckets (promotions, games, job alerts, newsletters) are only for bulk mail.\n\
         - Group with \"/\" sub-labels (e.g. \"Work/...\", \"Services/...\", \"Finance/...\").\n\
         - Reuse these existing labels verbatim when they fit: {}. If a sender's current label is too generic, propose a more precise one.\n\
         - Write every new label name in the language with BCP 47 tag \"{locale}\".\n\
         - If a whole domain belongs to one label, use the domain (e.g. linkedin.com). For personal mail domains like gmail.com or hotmail.com always use the exact address.\n\
         - One label per sender. Never invent senders. Leave out only what you cannot classify.\n\
         - Answer only with JSON shaped like {{\"rules\": [{{\"label\": \"Work/Acme\", \"senders\": [\"ana@acme.com\"]}}]}}.\n\n",
        labels.join(", ")
    );
    for s in samples {
        let subjects: Vec<String> = s.subjects.iter().map(|t| t.chars().take(SUBJECT_LENGTH).collect()).collect();
        prompt += &format!(
            "{} | {} <{}> | {} | {}\n",
            s.count,
            s.name,
            s.email,
            s.current.unwrap_or("-"),
            subjects.join(" ; ")
        );
    }
    prompt
}

fn acceptable(sender: &str, known: &HashSet<&str>) -> bool {
    valid_sender(sender)
        && (known.contains(sender)
            || (!sender.contains('@') && !PUBLIC_DOMAINS.contains(&sender) && known.iter().any(|k| k.ends_with(&format!("@{sender}")))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Mutex;

    fn sample(email: &str) -> Sample<'_> {
        Sample {
            email,
            name: "",
            count: 2,
            subjects: Vec::new(),
            current: None,
        }
    }

    #[test]
    fn only_accepts_what_it_was_shown() {
        let known: HashSet<&str> = ["jobs@linkedin.com", "friend@gmail.com"].into();
        assert!(acceptable("jobs@linkedin.com", &known));
        assert!(acceptable("linkedin.com", &known));
        assert!(!acceptable("gmail.com", &known));
        assert!(!acceptable("other@new.com", &known));
        assert!(!acceptable("jobs@linkedin.com) OR (x", &known));
    }

    #[test]
    fn batches_share_the_labels_they_create() {
        let samples = [sample("a@acme.com"), sample("b@acme.com"), sample("c@bank.example")];
        let prompts = Mutex::new(Vec::new());
        let ask = |p: &str| {
            prompts.lock().unwrap().push(p.to_string());
            Ok(match prompts.lock().unwrap().len() {
                1 => json!({ "rules": [{ "label": "Work/Acme", "senders": ["a@acme.com", "c@bank.example"] }] }),
                2 => json!({ "rules": [{ "label": "work/acme", "senders": ["b@acme.com", "a@acme.com"] }] }),
                _ => json!({ "rules": [{ "label": "Bad\"label", "senders": ["c@bank.example"] }] }),
            })
        };
        let steps = Mutex::new(Vec::new());
        let report = |p: Progress| steps.lock().unwrap().push((p.done, p.total));
        let rules = classify(ask, 1, "es", &samples, &["Finance".into()], &report).unwrap();
        let prompts = prompts.into_inner().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules["Work/Acme"], ["a@acme.com", "b@acme.com"]);
        assert!(prompts[1].contains("Finance, Work/Acme"));
        assert!(classify(|_| Ok(json!({})), 10, "es; x", &samples, &[], &report).is_err());
        assert_eq!(steps.into_inner().unwrap(), [(0, 3), (1, 3), (2, 3)]);
    }
}
