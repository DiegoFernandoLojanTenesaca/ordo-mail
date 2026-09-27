pub const SCOPES: &str = "https://www.googleapis.com/auth/gmail.modify https://www.googleapis.com/auth/gmail.settings.basic";
pub const GMAIL_API: &str = "https://gmail.googleapis.com/gmail/v1/users/me";
pub const AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

pub const CREDENTIALS_FILE: &str = "credentials.json";
pub const SETTINGS_FILE: &str = "settings.json";
pub const KEYRING_USER: &str = "google-refresh-token";

pub const THREADS: usize = 8;
pub const BATCH_SIZE: usize = 1000;
pub const PAGE_SIZE: usize = 500;
pub const COUNT_CAP: usize = 5000;
pub const MAX_RETRIES: u32 = 8;
pub const MAX_BACKOFF_SECS: u64 = 30;
pub const LOGIN_TIMEOUT_SECS: u64 = 300;
pub const LOGIN_POLL_MILLIS: u64 = 200;
pub const PROGRESS_EVERY: usize = 20;

pub const QUERY_UNLABELED: &str = "has:nouserlabels -from:me";
pub const QUERY_ALL: &str = "-from:me -in:chats";
pub const QUERY_SPAM: &str = "in:spam";
pub const MIN_MESSAGES_PER_SENDER: usize = 2;
pub const MAX_SENDERS_FOR_CLAUDE: usize = 300;
pub const SUBJECTS_PER_SENDER: usize = 3;
pub const SUBJECT_LENGTH: usize = 80;
pub const LABEL_LENGTH: usize = 40;
pub const LOCALE_LENGTH: usize = 16;

pub const PUBLIC_DOMAINS: &[&str] = &[
    "gmail.com", "googlemail.com", "hotmail.com", "hotmail.es", "outlook.com", "outlook.es",
    "live.com", "yahoo.com", "yahoo.es", "icloud.com", "protonmail.com", "proton.me",
];

pub const SPECIAL_FOLDERS: &[(&str, &str)] = &[
    ("SPAM", QUERY_SPAM),
    ("CATEGORY_PROMOTIONS", "category:promotions"),
    ("CATEGORY_SOCIAL", "category:social"),
];
