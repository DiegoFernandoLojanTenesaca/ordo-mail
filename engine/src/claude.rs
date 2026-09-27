use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::config::{LABEL_LENGTH, PUBLIC_DOMAINS, SUBJECT_LENGTH};
use crate::error::{Error, ErrorCode, Result};
use crate::settings::{ClaudeModel, valid_locale};
use crate::text::valid_sender;

const SCHEMA: &str = r#"{"type":"object","properties":{"rules":{"type":"array","items":{"type":"object","properties":{"label":{"type":"string"},"senders":{"type":"array","items":{"type":"string"}}},"required":["label","senders"]}}},"required":["rules"]}"#;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(crate) struct Sample<'a> {
    pub email: &'a str,
    pub name: &'a str,
    pub count: usize,
    pub subjects: Vec<&'a str>,
    pub current: Option<&'a str>,
}

pub(crate) fn classify(model: ClaudeModel, locale: &str, samples: &[Sample], existing: &[String]) -> Result<BTreeMap<String, Vec<String>>> {
    if !valid_locale(locale) {
        return Err(ErrorCode::InvalidLocale.into());
    }
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
         - One label per sender. Never invent senders. Leave out only what you cannot classify.\n\n",
        existing.join(", ")
    );
    for s in samples {
        let subjects: Vec<String> = s.subjects.iter().map(|t| t.chars().take(SUBJECT_LENGTH).collect()).collect();
        prompt += &format!("{} | {} <{}> | {} | {}\n", s.count, s.name, s.email, s.current.unwrap_or("-"), subjects.join(" ; "));
    }
    let answer = ask(model, &prompt)?;

    let known: HashSet<&str> = samples.iter().map(|s| s.email).collect();
    let mut seen = HashSet::new();
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for rule in answer["rules"].as_array().into_iter().flatten() {
        let label = rule["label"].as_str().unwrap_or_default().trim();
        if label.is_empty() || label.chars().count() > LABEL_LENGTH || label.contains(['=', '#', '"', '\n']) {
            continue;
        }
        let label = existing.iter().find(|e| e.eq_ignore_ascii_case(label)).cloned().unwrap_or_else(|| label.to_string());
        for sender in rule["senders"].as_array().into_iter().flatten().filter_map(Value::as_str) {
            let sender = sender.trim().to_lowercase();
            if acceptable(&sender, &known) && seen.insert(sender.clone()) {
                out.entry(label.clone()).or_default().push(sender);
            }
        }
    }
    Ok(out)
}

fn acceptable(sender: &str, known: &HashSet<&str>) -> bool {
    valid_sender(sender)
        && (known.contains(sender)
            || (!sender.contains('@') && !PUBLIC_DOMAINS.contains(&sender) && known.iter().any(|k| k.ends_with(&format!("@{sender}")))))
}

fn ask(model: ClaudeModel, prompt: &str) -> Result<Value> {
    let mut command = Command::new("claude");
    command
        .args(["-p", "--tools", "", "--strict-mcp-config", "--setting-sources", "", "--no-session-persistence",
               "--model", model.cli_name(), "--output-format", "json", "--json-schema", SCHEMA])
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().map_err(|_| Error::new(ErrorCode::ClaudeMissing))?;
    child.stdin.take().ok_or(ErrorCode::ClaudeFailed)?.write_all(prompt.as_bytes())?;
    let output = child.wait_with_output()?;
    let v: Value = serde_json::from_slice(&output.stdout)
        .map_err(|_| Error::with(ErrorCode::ClaudeFailed, String::from_utf8_lossy(&output.stderr)))?;
    if v["is_error"] == true || !v["structured_output"].is_object() {
        return Err(Error::with(ErrorCode::ClaudeFailed, &v["result"]));
    }
    Ok(v["structured_output"].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    #[ignore]
    fn claude_answers_live() {
        let samples = [
            Sample { email: "statements@bank.example", name: "Example Bank", count: 5, subjects: vec!["Your statement"], current: None },
            Sample { email: "ana.ruiz@acme.com", name: "Ana Ruiz", count: 9, subjects: vec!["Project update"], current: None },
        ];
        let rules = classify(ClaudeModel::Haiku, "es", &samples, &[]).unwrap();
        assert!(!rules.is_empty());
    }
}
