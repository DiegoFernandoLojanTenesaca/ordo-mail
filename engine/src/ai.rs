use std::io::Write;
use std::process::{Command, Stdio};
use std::{thread, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use ts_rs::TS;

use crate::config::{
    AI_TIMEOUT_SECS, ANTHROPIC_DEFAULT_MODEL, ANTHROPIC_FALLBACK_BETA, ANTHROPIC_FALLBACK_MODELS, ANTHROPIC_MAX_TOKENS, ANTHROPIC_URL,
    ANTHROPIC_VERSION, API_KEY_LENGTH, BATCH_LARGE, BATCH_MEDIUM, BATCH_SMALL, CLAUDE_CODE_MODELS, GROQ_URL, KEYRING_KEY_PREFIX,
    MAX_BACKOFF_SECS, MAX_RETRIES, MODEL_LENGTH, OLLAMA_URL,
};
use crate::error::{Error, ErrorCode, Result};
use crate::settings::Storage;
use crate::text::{extract_json, safe_base_url};

pub(crate) const SCHEMA: &str = r#"{"type":"object","additionalProperties":false,"required":["rules"],"properties":{"rules":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["label","senders"],"properties":{"label":{"type":"string"},"senders":{"type":"array","items":{"type":"string"}}}}}}}"#;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Provider {
    #[default]
    ClaudeCode,
    Anthropic,
    Groq,
    Ollama,
    OpenAiCompatible,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ProviderInfo {
    pub provider: Provider,
    pub default_model: Option<String>,
    pub default_base_url: Option<String>,
    pub needs_key: bool,
    pub accepts_key: bool,
    pub custom_url: bool,
    pub local: bool,
}

impl Provider {
    pub const ALL: [Provider; 5] = [
        Provider::ClaudeCode,
        Provider::Anthropic,
        Provider::Groq,
        Provider::Ollama,
        Provider::OpenAiCompatible,
    ];

    fn base_url(self) -> Option<&'static str> {
        match self {
            Provider::Anthropic => Some(ANTHROPIC_URL),
            Provider::Groq => Some(GROQ_URL),
            Provider::Ollama => Some(OLLAMA_URL),
            Provider::ClaudeCode | Provider::OpenAiCompatible => None,
        }
    }

    fn batch(self) -> usize {
        match self {
            Provider::ClaudeCode | Provider::Anthropic => BATCH_LARGE,
            Provider::Groq | Provider::OpenAiCompatible => BATCH_MEDIUM,
            Provider::Ollama => BATCH_SMALL,
        }
    }

    fn needs_key(self) -> bool {
        matches!(self, Provider::Anthropic | Provider::Groq)
    }

    fn accepts_key(self) -> bool {
        self.needs_key() || self == Provider::OpenAiCompatible
    }

    pub fn key_name(self) -> String {
        format!(
            "{KEYRING_KEY_PREFIX}{}",
            serde_json::to_value(self).unwrap_or_default().as_str().unwrap_or_default()
        )
    }

    pub fn info(self) -> ProviderInfo {
        ProviderInfo {
            provider: self,
            default_model: match self {
                Provider::ClaudeCode => CLAUDE_CODE_MODELS.first().map(|m| m.to_string()),
                Provider::Anthropic => Some(ANTHROPIC_DEFAULT_MODEL.into()),
                Provider::Groq | Provider::Ollama | Provider::OpenAiCompatible => None,
            },
            default_base_url: self.base_url().map(String::from),
            needs_key: self.needs_key(),
            accepts_key: self.accepts_key(),
            custom_url: matches!(self, Provider::Ollama | Provider::OpenAiCompatible),
            local: matches!(self, Provider::Ollama),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[serde(default)]
#[ts(export)]
pub struct AiSettings {
    pub provider: Provider,
    pub model: String,
    pub base_url: Option<String>,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            provider: Provider::default(),
            model: CLAUDE_CODE_MODELS.first().map(|m| m.to_string()).unwrap_or_default(),
            base_url: None,
        }
    }
}

impl AiSettings {
    pub fn validate(&self) -> Result<()> {
        if !self.model.is_empty() && !valid_model(&self.model) {
            return Err(Error::with(ErrorCode::InvalidModel, &self.model));
        }
        match self.base_url.as_deref().filter(|u| !u.is_empty()) {
            Some(url) if !safe_base_url(url) => Err(Error::with(ErrorCode::InvalidUrl, url)),
            _ => Ok(()),
        }
    }
}

pub fn valid_model(model: &str) -> bool {
    !model.is_empty()
        && model.len() <= MODEL_LENGTH
        && !model.starts_with('-')
        && model.chars().all(|c| c.is_ascii_alphanumeric() || "-._:/@".contains(c))
}

pub fn models(storage: &Storage, provider: Provider, base_url: Option<&str>) -> Result<Vec<String>> {
    Client::connect(storage, provider, base_url)?.models()
}

pub fn saved_keys(storage: &Storage) -> Result<Vec<Provider>> {
    let mut saved = Vec::new();
    for provider in Provider::ALL.into_iter().filter(|p| p.accepts_key()) {
        if storage.load_secret(&provider.key_name())?.is_some() {
            saved.push(provider);
        }
    }
    Ok(saved)
}

pub fn save_key(storage: &Storage, provider: Provider, key: &str) -> Result<()> {
    let key = key.trim();
    if !provider.accepts_key() || !valid_key(key) {
        return Err(ErrorCode::AiKeyRejected.into());
    }
    storage.save_secret(&provider.key_name(), key)
}

pub fn clear_key(storage: &Storage, provider: Provider) -> Result<()> {
    storage.clear_secret(&provider.key_name())
}

fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= API_KEY_LENGTH && key.chars().all(|c| c.is_ascii_graphic())
}

pub struct Client {
    provider: Provider,
    model: String,
    base_url: String,
    key: Option<String>,
    agent: ureq::Agent,
}

impl Client {
    pub fn connect(storage: &Storage, provider: Provider, base_url: Option<&str>) -> Result<Self> {
        let base_url = base_url
            .filter(|u| !u.is_empty())
            .map(String::from)
            .or_else(|| provider.base_url().map(String::from))
            .unwrap_or_default();
        if provider != Provider::ClaudeCode && !safe_base_url(&base_url) {
            return Err(Error::with(ErrorCode::InvalidUrl, &base_url));
        }
        let key = if provider.accepts_key() {
            storage.load_secret(&provider.key_name())?
        } else {
            None
        };
        if provider.needs_key() && key.is_none() {
            return Err(ErrorCode::AiKeyMissing.into());
        }
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(AI_TIMEOUT_SECS)))
            .build()
            .into();
        Ok(Self {
            provider,
            model: String::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            key,
            agent,
        })
    }

    pub fn load(storage: &Storage, settings: &AiSettings) -> Result<Self> {
        settings.validate()?;
        if !valid_model(&settings.model) {
            return Err(Error::with(ErrorCode::InvalidModel, &settings.model));
        }
        let mut client = Self::connect(storage, settings.provider, settings.base_url.as_deref())?;
        client.model = settings.model.clone();
        Ok(client)
    }

    pub fn batch(&self) -> usize {
        self.provider.batch()
    }

    pub fn models(&self) -> Result<Vec<String>> {
        let mut models: Vec<String> = match self.provider {
            Provider::ClaudeCode => return Ok(CLAUDE_CODE_MODELS.iter().map(|m| m.to_string()).collect()),
            Provider::Anthropic => self.get(&format!("{}/models?limit=100", self.base_url))?,
            _ => self.get(&format!("{}/models", self.base_url))?,
        }["data"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|m| m["id"].as_str().map(String::from))
            .collect();
        models.sort();
        Ok(models)
    }

    pub fn complete_json(&self, prompt: &str) -> Result<Value> {
        match self.provider {
            Provider::ClaudeCode => claude_code(&self.model, prompt),
            Provider::Anthropic => self.anthropic(prompt),
            _ => self.openai(prompt),
        }
    }

    fn headers(&self) -> Vec<(&'static str, String)> {
        match (self.provider, &self.key) {
            (Provider::Anthropic, Some(key)) => vec![("x-api-key", key.clone()), ("anthropic-version", ANTHROPIC_VERSION.into())],
            (_, Some(key)) => vec![("Authorization", format!("Bearer {key}"))],
            (_, None) => Vec::new(),
        }
    }

    fn anthropic(&self, prompt: &str) -> Result<Value> {
        let fallback = ANTHROPIC_FALLBACK_MODELS.iter().any(|m| self.model.starts_with(m));
        let mut body = json!({
            "model": self.model,
            "max_tokens": ANTHROPIC_MAX_TOKENS,
            "messages": [{ "role": "user", "content": prompt }],
            "output_config": { "format": { "type": "json_schema", "schema": serde_json::from_str::<Value>(SCHEMA)? } },
        });
        let mut headers = self.headers();
        if fallback {
            body["fallbacks"] = json!("default");
            headers.push(("anthropic-beta", ANTHROPIC_FALLBACK_BETA.into()));
        }
        let answer = self.post(&format!("{}/messages", self.base_url), &headers, &body)?;
        if answer["stop_reason"] == "refusal" {
            return Err(Error::with(ErrorCode::AiFailed, "refusal"));
        }
        let text = answer["content"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|b| b["type"] == "text")
            .and_then(|b| b["text"].as_str())
            .unwrap_or_default();
        extract_json(text).ok_or_else(|| Error::with(ErrorCode::AiFailed, text))
    }

    fn openai(&self, prompt: &str) -> Result<Value> {
        let body = json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": prompt }],
            "response_format": { "type": "json_object" },
            "temperature": 0,
        });
        let answer = self.post(&format!("{}/chat/completions", self.base_url), &self.headers(), &body)?;
        let text = answer["choices"][0]["message"]["content"].as_str().unwrap_or_default();
        extract_json(text).ok_or_else(|| Error::with(ErrorCode::AiFailed, text))
    }

    fn get(&self, url: &str) -> Result<Value> {
        let mut request = self.agent.get(url);
        for (name, value) in self.headers() {
            request = request.header(name, value);
        }
        let mut response = request.call().map_err(|e| Error::with(ErrorCode::AiUnavailable, e))?;
        let status = response.status().as_u16();
        let text = response.body_mut().read_to_string()?;
        check(status, &text)?;
        Ok(serde_json::from_str(&text)?)
    }

    fn post(&self, url: &str, headers: &[(&str, String)], body: &Value) -> Result<Value> {
        for attempt in 0..MAX_RETRIES {
            let mut request = self.agent.post(url);
            for (name, value) in headers {
                request = request.header(*name, value);
            }
            let mut response = request.send_json(body).map_err(|e| Error::with(ErrorCode::AiUnavailable, e))?;
            let status = response.status().as_u16();
            let wait = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());
            let text = response.body_mut().read_to_string()?;
            if status == 429 || status >= 500 {
                thread::sleep(Duration::from_secs(wait.unwrap_or(1 << attempt).min(MAX_BACKOFF_SECS)));
                continue;
            }
            check(status, &text)?;
            return Ok(serde_json::from_str(&text)?);
        }
        Err(Error::with(ErrorCode::AiFailed, "busy"))
    }
}

fn check(status: u16, text: &str) -> Result<()> {
    match status {
        401 | 403 => Err(ErrorCode::AiKeyRejected.into()),
        s if s >= 400 => Err(Error::with(ErrorCode::AiFailed, format!("{s} {text}"))),
        _ => Ok(()),
    }
}

fn claude_code(model: &str, prompt: &str) -> Result<Value> {
    let mut command = Command::new("claude");
    command
        .args([
            "-p",
            "--tools",
            "",
            "--strict-mcp-config",
            "--setting-sources",
            "",
            "--no-session-persistence",
            "--model",
            model,
            "--output-format",
            "json",
            "--json-schema",
            SCHEMA,
        ])
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().map_err(|_| Error::new(ErrorCode::AiUnavailable))?;
    child.stdin.take().ok_or(ErrorCode::AiFailed)?.write_all(prompt.as_bytes())?;
    let output = child.wait_with_output()?;
    let v: Value =
        serde_json::from_slice(&output.stdout).map_err(|_| Error::with(ErrorCode::AiFailed, String::from_utf8_lossy(&output.stderr)))?;
    if v["is_error"] == true || !v["structured_output"].is_object() {
        return Err(Error::with(ErrorCode::AiFailed, &v["result"]));
    }
    Ok(v["structured_output"].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read};
    use std::net::TcpListener;

    fn fake_server(reply: &'static str) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://127.0.0.1:{}/v1", listener.local_addr().unwrap().port());
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut head = String::new();
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
                    length = v.trim().parse().unwrap();
                }
                head += &line;
                if line == "\r\n" {
                    break;
                }
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{reply}",
                reply.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
            head + &String::from_utf8_lossy(&body)
        });
        (url.replace("127.0.0.1", "localhost"), handle)
    }

    fn client(provider: Provider, base_url: String, key: Option<&str>) -> Client {
        Client {
            provider,
            model: "test-model".into(),
            base_url,
            key: key.map(String::from),
            agent: ureq::Agent::config_builder().http_status_as_error(false).build().into(),
        }
    }

    #[test]
    fn openai_compatible_round_trip() {
        let (url, server) = fake_server(
            r#"{"choices":[{"message":{"content":"```json\n{\"rules\":[{\"label\":\"Work\",\"senders\":[\"a@x.com\"]}]}\n```"}}]}"#,
        );
        let answer = client(Provider::Groq, url, Some("secret")).complete_json("classify").unwrap();
        let request = server.join().unwrap();
        assert_eq!(answer["rules"][0]["label"], "Work");
        assert!(request.starts_with("POST /v1/chat/completions"));
        assert!(request.contains("Bearer secret") && request.contains("json_object") && request.contains("test-model"));
    }

    #[test]
    fn anthropic_round_trip() {
        let (url, server) = fake_server(r#"{"stop_reason":"end_turn","content":[{"type":"text","text":"{\"rules\":[]}"}]}"#);
        let answer = client(Provider::Anthropic, url, Some("sk-test")).complete_json("classify").unwrap();
        let request = server.join().unwrap();
        assert!(answer["rules"].as_array().unwrap().is_empty());
        assert!(request.starts_with("POST /v1/messages"));
        assert!(request.contains("x-api-key: sk-test") && request.contains("anthropic-version") && request.contains("json_schema"));
    }

    #[test]
    fn anthropic_refusal_is_an_error() {
        let (url, server) = fake_server(r#"{"stop_reason":"refusal","content":[]}"#);
        let error = client(Provider::Anthropic, url, Some("k")).complete_json("x").unwrap_err();
        server.join().unwrap();
        assert_eq!(error.code, ErrorCode::AiFailed);
    }

    #[test]
    fn models_are_listed() {
        let (url, server) = fake_server(r#"{"data":[{"id":"qwen3"},{"id":"llama3"}]}"#);
        let models = client(Provider::Ollama, url, None).models().unwrap();
        assert!(server.join().unwrap().starts_with("GET /v1/models"));
        assert_eq!(models, ["llama3", "qwen3"]);
    }

    #[test]
    fn settings_are_checked() {
        assert!(valid_model("llama-3.3-70b") && valid_model("openai/gpt-oss-120b") && valid_model("qwen3:8b"));
        assert!(!valid_model("--dangerously-skip-permissions") && !valid_model("a b") && !valid_model(""));
        let bad_url = AiSettings {
            base_url: Some("http://evil.example.com/v1".into()),
            ..AiSettings::default()
        };
        assert_eq!(bad_url.validate().unwrap_err().code, ErrorCode::InvalidUrl);
        assert!(
            valid_key("gsk_abc123")
                && !valid_key("gsk abc")
                && !valid_key("")
                && !valid_key(
                    "k
"
                )
        );
        assert_eq!(Provider::Groq.key_name(), "api-key-groq");
        assert_eq!(Provider::OpenAiCompatible.key_name(), "api-key-openAiCompatible");
    }

    #[test]
    #[ignore]
    fn claude_code_answers_live() {
        let answer = claude_code("haiku", "Answer only with JSON shaped like {\"rules\": []}.").unwrap();
        assert!(answer["rules"].is_array());
    }
}
