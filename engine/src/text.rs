use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

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
        .map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") })
        .collect()
}

pub fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let hex = b.get(i + 1..i + 3).and_then(|h| std::str::from_utf8(h).ok()).and_then(|h| u8::from_str_radix(h, 16).ok());
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
    from.rsplit_once('<').map_or(from, |(_, r)| r.split('>').next().unwrap_or_default()).trim().to_lowercase()
}

pub fn name_of(from: &str) -> String {
    from.rsplit_once('<').map_or("", |(n, _)| n).trim().trim_matches('"').trim().to_string()
}

pub fn valid_sender(q: &str) -> bool {
    q.contains('.') && q.chars().all(|c| c.is_ascii_alphanumeric() || "@._+-".contains(c))
}

pub fn label_query(name: &str) -> String {
    format!("label:\"{}\"", name.replace('"', ""))
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
}
