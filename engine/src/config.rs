pub const SCOPES: &str = "https://www.googleapis.com/auth/gmail.modify https://www.googleapis.com/auth/gmail.settings.basic";
pub const GMAIL_API: &str = "https://gmail.googleapis.com/gmail/v1/users/me";
pub const AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

pub const CREDENTIALS_FILE: &str = "credentials.json";
pub const SETTINGS_FILE: &str = "settings.json";
pub const KEYRING_TOKEN: &str = "google-refresh-token";
pub const KEYRING_KEY_PREFIX: &str = "api-key-";

pub const THREADS: usize = 8;
pub const BATCH_SIZE: usize = 1000;
pub const PAGE_SIZE: usize = 500;
pub const COUNT_CAP: usize = 5000;
pub const MAX_RETRIES: u32 = 8;
pub const MAX_BACKOFF_SECS: u64 = 30;
pub const LOGIN_TIMEOUT_SECS: u64 = 300;
pub const LOGIN_POLL_MILLIS: u64 = 200;
pub const PROGRESS_EVERY: usize = 20;

pub const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1";
pub const ANTHROPIC_VERSION: &str = "2023-06-01";
pub const ANTHROPIC_DEFAULT_MODEL: &str = "claude-opus-5";
pub const ANTHROPIC_MAX_TOKENS: u32 = 16000;
pub const ANTHROPIC_FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";
pub const ANTHROPIC_FALLBACK_MODELS: &[&str] = &["claude-opus-5", "claude-fable", "claude-mythos"];
pub const GROQ_URL: &str = "https://api.groq.com/openai/v1";
pub const OLLAMA_URL: &str = "http://localhost:11434/v1";
pub const CLAUDE_CODE_MODELS: &[&str] = &["sonnet", "haiku", "opus"];
pub const LOCAL_HOSTS: &[&str] = &["localhost", "127.0.0.1", "[::1]"];
pub const AI_TIMEOUT_SECS: u64 = 600;
pub const BATCH_LARGE: usize = 300;
pub const BATCH_MEDIUM: usize = 60;
pub const BATCH_SMALL: usize = 30;
pub const MODEL_LENGTH: usize = 100;
pub const API_KEY_LENGTH: usize = 512;

pub const UNSUBSCRIBE_TIMEOUT_SECS: u64 = 30;
pub const UNSUBSCRIBE_SAMPLES: usize = 5;
pub const ONE_CLICK_KEY: &str = "List-Unsubscribe";
pub const ONE_CLICK_VALUE: &str = "One-Click";

pub const QUERY_UNLABELED: &str = "has:nouserlabels -from:me";
pub const QUERY_ALL: &str = "-from:me -in:chats";
pub const QUERY_SPAM: &str = "in:spam";
pub const QUERY_ANYWHERE: &str = "in:anywhere";
pub const MIN_MESSAGES_PER_SENDER: usize = 2;
pub const MAX_SENDERS_FOR_AI: usize = 300;
pub const SUBJECTS_PER_SENDER: usize = 3;
pub const SUBJECT_LENGTH: usize = 80;
pub const LABEL_LENGTH: usize = 40;
pub const LOCALE_LENGTH: usize = 16;

pub const PUBLIC_DOMAINS: &[&str] = &[
    "gmail.com",
    "googlemail.com",
    "hotmail.com",
    "hotmail.es",
    "outlook.com",
    "outlook.es",
    "live.com",
    "yahoo.com",
    "yahoo.es",
    "icloud.com",
    "protonmail.com",
    "proton.me",
];

pub const SPECIAL_FOLDERS: &[(&str, &str)] = &[
    ("SPAM", QUERY_SPAM),
    ("CATEGORY_PROMOTIONS", "category:promotions"),
    ("CATEGORY_SOCIAL", "category:social"),
];
