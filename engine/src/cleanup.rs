use crate::config::SPECIAL_FOLDERS;
use crate::error::{Error, ErrorCode, Result};
use crate::gmail::{Gmail, parallel};
use crate::model::{CleanupItem, Phase, Progress, Report};
use crate::settings::Settings;
use crate::text::{label_query, valid_sender};

struct Source {
    id: String,
    name: String,
    query: String,
    protected: bool,
    special: bool,
}

fn sources(gmail: &Gmail, settings: &Settings) -> Result<Vec<Source>> {
    let special = SPECIAL_FOLDERS.iter().map(|(id, query)| Source {
        id: id.to_string(),
        name: id.to_string(),
        query: query.to_string(),
        protected: false,
        special: true,
    });
    let own = gmail.user_labels()?.into_iter().map(|(id, name)| Source {
        query: label_query(&name),
        protected: settings.is_protected(&name),
        special: false,
        id,
        name,
    });
    Ok(special.chain(own).collect())
}

fn query(source: &Source, days: u32, settings: &Settings) -> String {
    let mut q = source.query.clone();
    if days > 0 {
        q += &format!(" older_than:{days}d");
    }
    for p in &settings.protected {
        q += &format!(" -{}", label_query(p));
    }
    q
}

pub fn items(gmail: &Gmail, settings: &Settings, days: u32) -> Result<Vec<CleanupItem>> {
    let sources = sources(gmail, settings)?;
    let counts = parallel(&sources, |s| {
        if s.protected {
            Ok((0, true))
        } else {
            gmail.count(&query(s, days, settings))
        }
    });
    sources
        .into_iter()
        .zip(counts)
        .map(|(s, count)| {
            let (total, complete) = count?;
            Ok(CleanupItem {
                id: s.id,
                name: s.name,
                total,
                complete,
                protected: s.protected,
                special: s.special,
            })
        })
        .collect()
}

pub fn clean(gmail: &Gmail, settings: &Settings, ids: &[String], days: u32, report: Report) -> Result<usize> {
    let chosen: Vec<Source> = sources(gmail, settings)?
        .into_iter()
        .filter(|s| ids.contains(&s.id) && !s.protected)
        .collect();
    let mut total = 0;
    for (i, s) in chosen.iter().enumerate() {
        report(Progress::about(Phase::Cleaning, &s.name, i, chosen.len()));
        let messages = gmail.search(&query(s, days, settings), usize::MAX)?;
        gmail.modify(&messages, &["TRASH"], &[])?;
        total += messages.len();
    }
    report(Progress::new(Phase::Done, chosen.len(), chosen.len()));
    Ok(total)
}

pub fn trash_sender(gmail: &Gmail, settings: &Settings, email: &str) -> Result<usize> {
    if !valid_sender(email) {
        return Err(Error::with(ErrorCode::InvalidSender, email));
    }
    let source = Source {
        id: email.to_string(),
        name: email.to_string(),
        query: format!("from:{email}"),
        protected: false,
        special: false,
    };
    let messages = gmail.search(&query(&source, 0, settings), usize::MAX)?;
    gmail.modify(&messages, &["TRASH"], &[])?;
    Ok(messages.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_labels_are_always_excluded() {
        let settings = Settings {
            protected: vec!["Bank".into()],
            ..Settings::default()
        };
        let s = Source {
            id: "X".into(),
            name: "X".into(),
            query: "category:promotions".into(),
            protected: false,
            special: true,
        };
        assert_eq!(query(&s, 30, &settings), "category:promotions older_than:30d -label:\"Bank\"");
        assert_eq!(query(&s, 0, &Settings::default()), "category:promotions");
    }
}
