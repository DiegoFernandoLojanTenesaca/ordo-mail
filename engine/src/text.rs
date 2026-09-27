use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::Value;
use url::{Host, Url};

use crate::config::{LOCAL_HOSTS, ONE_CLICK_KEY, ONE_CLICK_VALUE};
use crate::model::Unsubscribe;

pub fn b64(bytes: impl AsRef<[u8]>) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn random(bytes: usize) -> String {
    let mut b = vec![0; bytes];
    getrandom::fill(&mut b).expect("system randomness unavailable");
    b64(b)
}

pub fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

pub fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let hex = b
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (b[i], hex) {
            (b'%', Some(x)) => {
                out.push(x);
                i += 2;
            }
            (b'+', _) => out.push(b' '),
            (c, _) => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn escape_html(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '<' => "&lt;".into(),
            '>' => "&gt;".into(),
            '&' => "&amp;".into(),
            '"' => "&quot;".into(),
            '\'' => "&#39;".into(),
            c => c.to_string(),
        })
        .collect()
}

pub fn email_of(from: &str) -> String {
    from.rsplit_once('<')
        .map_or(from, |(_, r)| r.split('>').next().unwrap_or_default())
        .trim()
        .to_lowercase()
}

pub fn name_of(from: &str) -> String {
    from.rsplit_once('<')
        .map_or("", |(n, _)| n)
        .trim()
        .trim_matches('"')
        .trim()
        .to_string()
}

pub fn valid_sender(q: &str) -> bool {
    q.contains('.') && q.chars().all(|c| c.is_ascii_alphanumeric() || "@._+-".contains(c))
}

pub fn label_query(name: &str) -> String {
    format!("label:\"{}\"", name.replace('"', ""))
}

pub fn safe_web_url(u: &str) -> bool {
    Url::parse(u).is_ok_and(|p| {
        p.scheme() == "https"
            && matches!(p.host(), Some(Host::Domain(d)) if d.contains('.') && !LOCAL_HOSTS.contains(&d) && !d.ends_with(".local"))
    })
}

pub fn safe_base_url(u: &str) -> bool {
    Url::parse(u).is_ok_and(|p| match p.scheme() {
        "https" => p.host().is_some(),
        "http" => p.host_str().is_some_and(|h| LOCAL_HOSTS.contains(&h)),
        _ => false,
    })
}

pub fn unsubscribe_targets(header: &str) -> (Option<String>, Option<String>) {
    let targets: Vec<&str> = header
        .split(',')
        .filter_map(|t| t.trim().strip_prefix('<')?.strip_suffix('>'))
        .collect();
    let web = targets.iter().find(|t| t.starts_with("https://")).map(|t| t.to_string());
    let mail = targets.iter().find(|t| t.starts_with("mailto:")).map(|t| t.to_string());
    (web, mail)
}

pub fn unsubscribe_kind(header: &str, post: &str) -> Unsubscribe {
    let (web, mail) = unsubscribe_targets(header);
    let one_click = post.contains(&format!("{ONE_CLICK_KEY}={ONE_CLICK_VALUE}"));
    match (web.filter(|w| safe_web_url(w)), mail) {
        (Some(_), _) if one_click => Unsubscribe::OneClick,
        (Some(_), _) => Unsubscribe::Link,
        (None, Some(_)) => Unsubscribe::Mail,
        (None, None) => Unsubscribe::None,
    }
}

pub fn extract_json(s: &str) -> Option<Value> {
    serde_json::from_str(s)
        .ok()
        .or_else(|| serde_json::from_str(&s[s.find('{')?..=s.rfind('}')?]).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_encoding_round_trips() {
        assert_eq!(decode(&encode("4/0Ab c+ñ")), "4/0Ab c+ñ");
        assert_eq!(decode("4%2F0Ab%zz+x"), "4/0Ab%zz x");
    }

    #[test]
    fn from_header_is_split() {
        assert_eq!(email_of("Ana Ruiz <Ana@Acme.com>"), "ana@acme.com");
        assert_eq!(email_of(" solo@x.com "), "solo@x.com");
        assert_eq!(name_of("\"Ana Ruiz\" <ana@acme.com>"), "Ana Ruiz");
        assert_eq!(name_of("solo@x.com"), "");
    }

    #[test]
    fn senders_never_carry_operators() {
        assert!(valid_sender("jobs@linkedin.com") && valid_sender("linkedin.com"));
        assert!(!valid_sender("a@b.com) OR (x") && !valid_sender("nodot"));
        assert_eq!(label_query("Bank \"X\""), "label:\"Bank X\"");
        assert_eq!(escape_html("<b>'x'&</b>"), "&lt;b&gt;&#39;x&#39;&amp;&lt;/b&gt;");
    }
    #[test]
    fn unsubscribe_headers_are_understood() {
        let header = "<mailto:off@news.example.com?subject=stop>, <https://news.example.com/u/42>";
        assert_eq!(
            unsubscribe_targets(header),
            (
                Some("https://news.example.com/u/42".into()),
                Some("mailto:off@news.example.com?subject=stop".into())
            )
        );
        assert_eq!(unsubscribe_kind(header, "List-Unsubscribe=One-Click"), Unsubscribe::OneClick);
        assert_eq!(unsubscribe_kind(header, ""), Unsubscribe::Link);
        assert_eq!(unsubscribe_kind("<mailto:off@x.com>", ""), Unsubscribe::Mail);
        assert_eq!(
            unsubscribe_kind("<https://192.168.1.1/u>", "List-Unsubscribe=One-Click"),
            Unsubscribe::None
        );
        assert_eq!(unsubscribe_kind("", ""), Unsubscribe::None);
    }

    #[test]
    fn only_public_https_links_are_safe() {
        assert!(safe_web_url("https://news.example.com/u?id=1"));
        assert!(!safe_web_url("http://news.example.com/u") && !safe_web_url("https://localhost/u"));
        assert!(!safe_web_url("https://10.0.0.1/u") && !safe_web_url("https://[::1]/u") && !safe_web_url("https://printer.local/"));
        assert!(safe_base_url("https://api.groq.com/openai/v1") && safe_base_url("http://localhost:11434/v1"));
        assert!(!safe_base_url("http://api.example.com/v1") && !safe_base_url("file:///etc/passwd"));
    }

    #[test]
    fn json_is_found_inside_fences() {
        assert_eq!(
            extract_json(
                "```json
{\"a\": 1}
```"
            )
            .unwrap()["a"],
            1
        );
        assert_eq!(extract_json("{\"a\": 2}").unwrap()["a"], 2);
        assert!(extract_json("no json here").is_none());
    }
}
