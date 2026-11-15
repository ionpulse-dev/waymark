//! A fake GitHub for the personal server tests (see `specs/foundation/personal-server/test.md`).
//!
//! One local HTTP server plays four parts: the GitHub sign-in (`/login/oauth/...`), the GitHub API
//! (`/user`, `/user/emails`, the App token), a Git host (smart HTTP through `git http-backend`,
//! with the App token as the only credential), and a host for client documents.

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Account {
    pub login: String,
    pub name: Option<String>,
    /// (address, verified, primary)
    pub emails: Vec<(String, bool, bool)>,
}

#[derive(Default)]
pub struct Inner {
    /// The sign-in code that a test hands over, and the account it stands for.
    pub accounts: HashMap<String, Account>,
    /// The token of the current App installation token. An older one is refused by the Git host.
    pub valid_token: Option<String>,
    pub tokens_issued: usize,
    pub git_refused: usize,
    /// A delay before the Git host answers, in milliseconds (a slow clone).
    pub git_delay_ms: u64,
    /// The client document served at `/client/doc.json`.
    pub client_doc: Option<Value>,
}

pub struct FakeGithub {
    pub base: String,
    pub git_root: PathBuf,
    pub inner: Arc<Mutex<Inner>>,
}

type Shared = (Arc<Mutex<Inner>>, PathBuf);

impl FakeGithub {
    pub fn start(git_root: PathBuf) -> FakeGithub {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let inner = Arc::new(Mutex::new(Inner::default()));
        let shared: Shared = (inner.clone(), git_root.clone());
        let app = Router::new()
            .route("/login/oauth/access_token", post(access_token))
            .route("/user", get(user))
            .route("/user/emails", get(emails))
            .route("/app/installations/{id}/access_tokens", post(app_token))
            .route("/client/doc.json", get(client_doc))
            .route("/git/{*rest}", get(git).post(git))
            .with_state(shared);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                let l = tokio::net::TcpListener::from_std(listener).unwrap();
                axum::serve(l, app).await.unwrap();
            });
        });
        FakeGithub {
            base: format!("http://127.0.0.1:{port}"),
            git_root,
            inner,
        }
    }

    pub fn set_account(&self, code: &str, account: Account) {
        self.inner
            .lock()
            .unwrap()
            .accounts
            .insert(code.to_string(), account);
    }

    /// Make every token that was issued until now invalid at the Git host.
    pub fn expire_tokens(&self) {
        self.inner.lock().unwrap().valid_token = None;
    }

    pub fn tokens_issued(&self) -> usize {
        self.inner.lock().unwrap().tokens_issued
    }

    pub fn git_refused(&self) -> usize {
        self.inner.lock().unwrap().git_refused
    }

    pub fn set_git_delay_ms(&self, ms: u64) {
        self.inner.lock().unwrap().git_delay_ms = ms;
    }

    pub fn set_client_doc(&self, doc: Value) {
        self.inner.lock().unwrap().client_doc = Some(doc);
    }

    /// The URL of a bare repository in `git_root`, such as `memory.git`.
    pub fn repo_url(&self, name: &str) -> String {
        format!("{}/git/{name}", self.base)
    }
}

async fn access_token(
    State((inner, _)): State<Shared>,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let code = f.get("code").cloned().unwrap_or_default();
    if inner.lock().unwrap().accounts.contains_key(&code) {
        Json(json!({"access_token": format!("gho_{code}"), "token_type": "bearer"})).into_response()
    } else {
        Json(json!({"error": "bad_verification_code"})).into_response()
    }
}

fn account_of(inner: &Inner, headers: &HeaderMap) -> Option<Account> {
    let v = headers.get("authorization")?.to_str().ok()?;
    let token = v.strip_prefix("Bearer ")?;
    let code = token.strip_prefix("gho_")?;
    inner.accounts.get(code).cloned()
}

async fn user(State((inner, _)): State<Shared>, headers: HeaderMap) -> Response {
    match account_of(&inner.lock().unwrap(), &headers) {
        Some(a) => Json(json!({"login": a.login, "name": a.name})).into_response(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn emails(State((inner, _)): State<Shared>, headers: HeaderMap) -> Response {
    match account_of(&inner.lock().unwrap(), &headers) {
        Some(a) => Json(
            a.emails
                .iter()
                .map(|(e, v, p)| json!({"email": e, "verified": v, "primary": p}))
                .collect::<Vec<_>>(),
        )
        .into_response(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn app_token(State((inner, _)): State<Shared>, headers: HeaderMap) -> Response {
    // The statement of the App is a signed token with three parts.
    let ok = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|t| t.split('.').count() == 3);
    if !ok {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let mut g = inner.lock().unwrap();
    g.tokens_issued += 1;
    let token = format!("ghs_fake{}", g.tokens_issued);
    g.valid_token = Some(token.clone());
    (
        StatusCode::CREATED,
        Json(json!({"token": token, "expires_at": "2099-01-01T00:00:00Z"})),
    )
        .into_response()
}

async fn client_doc(State((inner, _)): State<Shared>) -> Response {
    match inner.lock().unwrap().client_doc.clone() {
        Some(d) => Json(d).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// The Git host: `git http-backend` behind the App token.
async fn git(
    State((inner, root)): State<Shared>,
    Path(rest): Path<String>,
    method: axum::http::Method,
    uri: axum::http::Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    use base64::Engine;
    let (valid, delay) = {
        let g = inner.lock().unwrap();
        (g.valid_token.clone(), g.git_delay_ms)
    };
    let given = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Basic "))
        .and_then(|b| {
            base64::engine::general_purpose::STANDARD
                .decode(b.trim())
                .ok()
        })
        .and_then(|b| String::from_utf8(b).ok());
    let allowed = match (&valid, &given) {
        (Some(v), Some(g)) => g == &format!("x-access-token:{v}"),
        _ => false,
    };
    if !allowed {
        inner.lock().unwrap().git_refused += 1;
        let mut r = StatusCode::UNAUTHORIZED.into_response();
        r.headers_mut()
            .insert("www-authenticate", "Basic realm=\"git\"".parse().unwrap());
        return r;
    }
    if delay > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
    let path_info = format!("/{rest}");
    let query = uri.query().unwrap_or("").to_string();
    let ctype = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let encoding = headers
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let protocol = headers
        .get("git-protocol")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let method = method.as_str().to_string();
    tokio::task::spawn_blocking(move || {
        let cgi = Cgi {
            method,
            path_info,
            query,
            content_type: ctype,
            encoding,
            protocol,
        };
        run_backend(&root, &cgi, &body)
    })
    .await
    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// The parts of a request that the CGI program of Git needs.
struct Cgi {
    method: String,
    path_info: String,
    query: String,
    content_type: String,
    encoding: String,
    protocol: String,
}

fn run_backend(root: &std::path::Path, cgi: &Cgi, body: &[u8]) -> Response {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let exec = Command::new("git")
        .arg("--exec-path")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let mut child = Command::new(format!("{exec}/git-http-backend"))
        .env("GIT_PROJECT_ROOT", root)
        .env("GIT_HTTP_EXPORT_ALL", "1")
        .env("REQUEST_METHOD", &cgi.method)
        .env("PATH_INFO", &cgi.path_info)
        .env("QUERY_STRING", &cgi.query)
        .env("CONTENT_TYPE", &cgi.content_type)
        .env("CONTENT_LENGTH", body.len().to_string())
        .env("HTTP_CONTENT_ENCODING", &cgi.encoding)
        .env("HTTP_GIT_PROTOCOL", &cgi.protocol)
        .env("REMOTE_USER", "x-access-token")
        .env("REMOTE_ADDR", "127.0.0.1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("run git http-backend");
    let mut stdin = child.stdin.take().unwrap();
    let data = body.to_vec();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&data);
    });
    let out = child.wait_with_output().unwrap();
    let _ = writer.join();
    let raw = out.stdout;
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| (p, 4))
        .or_else(|| raw.windows(2).position(|w| w == b"\n\n").map(|p| (p, 2)));
    let Some((pos, skip)) = split else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let head = String::from_utf8_lossy(&raw[..pos]).to_string();
    let payload = raw[pos + skip..].to_vec();
    let mut status = StatusCode::OK;
    let mut resp_headers: Vec<(String, String)> = Vec::new();
    for line in head.lines() {
        if let Some((k, v)) = line.split_once(':') {
            if k.eq_ignore_ascii_case("status") {
                if let Some(code) = v
                    .trim()
                    .split(' ')
                    .next()
                    .and_then(|c| c.parse::<u16>().ok())
                {
                    status = StatusCode::from_u16(code).unwrap_or(StatusCode::OK);
                }
            } else {
                resp_headers.push((k.trim().to_string(), v.trim().to_string()));
            }
        }
    }
    let mut r = (status, payload).into_response();
    for (k, v) in resp_headers {
        if let (Ok(k), Ok(v)) = (
            axum::http::HeaderName::from_bytes(k.as_bytes()),
            axum::http::HeaderValue::from_str(&v),
        ) {
            r.headers_mut().insert(k, v);
        }
    }
    r
}
