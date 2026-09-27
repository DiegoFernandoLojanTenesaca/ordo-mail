use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};
use std::{fs, thread};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use ts_rs::TS;

use crate::config::{AUTHORIZE_URL, LOGIN_POLL_MILLIS, LOGIN_TIMEOUT_SECS, SCOPES, TOKEN_URL};
use crate::error::{Error, ErrorCode, Result};
use crate::settings::Storage;
use crate::text::{b64, decode, encode, escape_html, random};

pub struct Client {
    id: String,
    secret: String,
}

impl Client {
    pub fn load(storage: &Storage) -> Result<Self> {
        let text = fs::read_to_string(storage.credentials()).map_err(|_| Error::new(ErrorCode::MissingCredentials))?;
        Self::parse(&text)
    }

    fn parse(text: &str) -> Result<Self> {
        let v: Value = serde_json::from_str(text).map_err(|_| Error::new(ErrorCode::InvalidCredentials))?;
        let installed = &v["installed"];
        let id = installed["client_id"].as_str().ok_or(ErrorCode::NotDesktopClient)?;
        Ok(Self { id: id.into(), secret: installed["client_secret"].as_str().unwrap_or_default().into() })
    }
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct LoginPage {
    pub title: String,
    pub message: String,
}

impl LoginPage {
    fn html(&self) -> String {
        format!(
            "<!doctype html><meta charset=utf-8><title>{title}</title>\
             <body style='margin:0;display:grid;place-items:center;height:100vh;color-scheme:light dark;\
             background:Canvas;color:CanvasText;font-family:system-ui,sans-serif;text-align:center'>\
             <div><div style='font-size:56px'>&#10003;</div><h1 style='font-weight:400'>{title}</h1><p>{message}</p></div>",
            title = escape_html(&self.title),
            message = escape_html(&self.message),
        )
    }
}

pub fn save_credentials(storage: &Storage, text: &str) -> Result<()> {
    Client::parse(text)?;
    storage.write(&storage.credentials(), text)
}

pub fn refresh(agent: &ureq::Agent, client: &Client, refresh_token: &str) -> Result<String> {
    let form = [("grant_type", "refresh_token"), ("refresh_token", refresh_token), ("client_id", &client.id), ("client_secret", &client.secret)];
    Ok(request_token(agent, &form)?["access_token"].as_str().unwrap_or_default().to_string())
}

pub fn login(agent: &ureq::Agent, client: &Client, page: &LoginPage) -> Result<(String, String)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let redirect = format!("http://127.0.0.1:{}", listener.local_addr()?.port());
    let verifier = random(32);
    let state = random(16);
    let url = format!(
        "{AUTHORIZE_URL}?response_type=code&access_type=offline&prompt=select_account%20consent\
         &client_id={}&redirect_uri={}&scope={}&state={state}&code_challenge_method=S256&code_challenge={}",
        encode(&client.id), encode(&redirect), encode(SCOPES), b64(Sha256::digest(&verifier)),
    );
    open::that(&url)?;

    let deadline = Instant::now() + Duration::from_secs(LOGIN_TIMEOUT_SECS);
    loop {
        let mut stream = match listener.accept() {
            Ok((s, _)) => s,
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                if Instant::now() > deadline {
                    return Err(ErrorCode::LoginTimeout.into());
                }
                thread::sleep(Duration::from_millis(LOGIN_POLL_MILLIS));
                continue;
            }
            Err(e) => return Err(e.into()),
        };
        stream.set_nonblocking(false)?;
        let mut buf = [0; 8192];
        let n = stream.read(&mut buf)?;
        let request = String::from_utf8_lossy(&buf[..n]);
        let path = request.split_whitespace().nth(1).unwrap_or_default();
        let query: HashMap<&str, String> = path.split_once('?').map_or("", |(_, q)| q)
            .split('&').filter_map(|p| p.split_once('=')).map(|(k, v)| (k, decode(v))).collect();
        if !query.contains_key("code") && !query.contains_key("error") {
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
            continue;
        }
        let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{}", page.html()).as_bytes());
        if query.get("state") != Some(&state) {
            return Err(ErrorCode::LoginMismatch.into());
        }
        let code = query.get("code").ok_or_else(|| Error::with(ErrorCode::LoginDenied, query.get("error").map_or("", |e| e)))?;
        let form = [("grant_type", "authorization_code"), ("code", code), ("code_verifier", &verifier),
                    ("redirect_uri", &redirect), ("client_id", &client.id), ("client_secret", &client.secret)];
        let v = request_token(agent, &form)?;
        let access = v["access_token"].as_str().unwrap_or_default().to_string();
        let refresh = v["refresh_token"].as_str().ok_or(ErrorCode::NoRefreshToken)?.to_string();
        return Ok((access, refresh));
    }
}

fn request_token(agent: &ureq::Agent, form: &[(&str, &str)]) -> Result<Value> {
    let v: Value = agent.post(TOKEN_URL).send_form(form.iter().copied())?.body_mut().read_json()?;
    if v["access_token"].is_string() { Ok(v) } else { Err(Error::with(ErrorCode::TokenRejected, &v["error"])) }
}
