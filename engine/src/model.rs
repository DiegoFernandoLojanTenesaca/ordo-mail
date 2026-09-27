use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Action {
    #[default]
    Label,
    Archive,
    Trash,
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
pub struct Status {
    pub has_credentials: bool,
    pub account: Option<String>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct Summary {
    pub unlabeled: usize,
    pub unlabeled_complete: bool,
    pub labels: usize,
    pub rules: usize,
    pub spam: usize,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub total: u32,
    pub protected: bool,
    pub rules: Vec<Rule>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct Rule {
    pub filter_id: String,
    pub sender: String,
    pub action: Action,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct Sender {
    pub email: String,
    pub count: usize,
    pub current: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct Group {
    pub label: String,
    pub is_new: bool,
    pub action: Action,
    pub senders: Vec<Sender>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct Proposal {
    pub read: usize,
    pub failed: usize,
    pub groups: Vec<Group>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct CleanupItem {
    pub id: String,
    pub name: String,
    pub total: usize,
    pub complete: bool,
    pub protected: bool,
    pub special: bool,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Phase {
    Searching,
    Reading,
    Thinking,
    Applying,
    Cleaning,
    Done,
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
pub struct Progress {
    pub phase: Phase,
    pub detail: Option<String>,
    pub done: usize,
    pub total: usize,
}

impl Progress {
    pub fn new(phase: Phase, done: usize, total: usize) -> Self {
        Self { phase, detail: None, done, total }
    }

    pub fn about(phase: Phase, detail: impl Into<String>, done: usize, total: usize) -> Self {
        Self { phase, detail: Some(detail.into()), done, total }
    }
}

pub type Report<'a> = &'a (dyn Fn(Progress) + Sync);
