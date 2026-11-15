//! The personal server under test: the real binary in `--mode personal-server`, with a fake GitHub
//! (see `specs/foundation/personal-server/test.md`).

use super::fake_github::{Account, FakeGithub};
use super::git::git;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const SIGNING_KEY: &str = "test-signing-key-0123456789abcdef-test";
pub const APP_SECRET_MARKER: &str = "test-client-secret-MARKER-8842";
pub const DOC_MARKER: &str = "document-marker-text-5531";

/// A throwaway RSA key for the fake GitHub App, made for each test run. It uses `openssl`, or
/// `ssh-keygen` when `openssl` is not installed.
pub fn rsa_key() -> String {
    if let Ok(out) = Command::new("openssl").args(["genrsa", "2048"]).output() {
        if out.status.success() {
            return String::from_utf8(out.stdout).unwrap();
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("key");
    let status = Command::new("ssh-keygen")
        .args(["-q", "-t", "rsa", "-b", "2048", "-m", "PEM", "-N", "", "-f"])
        .arg(&file)
        .status()
        .expect("openssl or ssh-keygen is needed to make a test key");
    assert!(status.success(), "ssh-keygen failed");
    std::fs::read_to_string(&file).unwrap()
}

pub struct Env {
    pub root: PathBuf,
    pub fake: FakeGithub,
    pub key: String,
}

impl Env {
    /// A fake GitHub with a bare memory repository `memory.git` (with `init` done unless `empty`).
    pub fn new(empty: bool) -> Env {
        let root = tempfile::Builder::new()
            .prefix("waymark-ps-")
            .tempdir()
            .unwrap()
            .keep();
        let git_root = root.join("git");
        std::fs::create_dir_all(&git_root).unwrap();
        let bare = git_root.join("memory.git");
        std::fs::create_dir_all(&bare).unwrap();
        git(&bare, &["init", "--bare", "--initial-branch=main", "-q"]);
        git(&bare, &["config", "http.receivepack", "true"]);
        if !empty {
            let seed = root.join("seed");
            git(&root, &["clone", "-q", bare.to_str().unwrap(), "seed"]);
            git(&seed, &["checkout", "-q", "-b", "main"]);
            waymark::memrepo::init(&seed, "Test", "test").unwrap();
            git(&seed, &["add", "-A"]);
            git(
                &seed,
                &[
                    "-c",
                    "user.name=Seed",
                    "-c",
                    "user.email=seed@example.org",
                    "commit",
                    "-q",
                    "-m",
                    "Initial",
                ],
            );
            git(&seed, &["push", "-q", "-u", "origin", "main"]);
        }
        Env {
            root,
            fake: FakeGithub::start(git_root),
            key: rsa_key(),
        }
    }

    pub fn owner_account() -> Account {
        Account {
            login: "owner".into(),
            name: Some("Olivia Owner".into()),
            emails: vec![("olivia@example.org".into(), true, true)],
        }
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[derive(Default)]
pub struct Options {
    pub extra_env: Vec<(String, String)>,
    pub args: Vec<String>,
    /// Do not wait until the server answers (for a start that must fail).
    pub no_wait: bool,
}

pub struct Server {
    pub child: Child,
    pub port: u16,
    pub base: String,
    pub tmp: PathBuf,
    pub config: PathBuf,
    pub out: Arc<Mutex<Vec<String>>>,
}

pub fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

impl Server {
    /// The environment of a correct start. A test may change it.
    pub fn env_for(env: &Env, port: u16, tmp: &Path, config: &Path) -> Vec<(String, String)> {
        let base = format!("http://127.0.0.1:{port}");
        vec![
            ("PORT".into(), port.to_string()),
            ("WAYMARK_PUBLIC_URL".into(), base),
            ("WAYMARK_REPO".into(), env.fake.repo_url("memory.git")),
            ("WAYMARK_OWNER".into(), "owner".into()),
            ("WAYMARK_SIGNING_KEY".into(), SIGNING_KEY.into()),
            ("WAYMARK_GITHUB_APP_ID".into(), "123".into()),
            ("WAYMARK_GITHUB_INSTALLATION_ID".into(), "456".into()),
            ("WAYMARK_GITHUB_APP_KEY".into(), env.key.clone()),
            ("WAYMARK_GITHUB_CLIENT_ID".into(), "client-id".into()),
            (
                "WAYMARK_GITHUB_CLIENT_SECRET".into(),
                APP_SECRET_MARKER.into(),
            ),
            ("WAYMARK_GITHUB_URL".into(), env.fake.base.clone()),
            ("WAYMARK_GITHUB_API_URL".into(), env.fake.base.clone()),
            ("TMPDIR".into(), tmp.display().to_string()),
            ("WAYMARK_SYSTEM_CONFIG".into(), config.display().to_string()),
            ("WAYMARK_MODE".into(), "personal-server".into()),
        ]
    }

    pub fn start(env: &Env) -> Server {
        Server::start_with(env, Options::default())
    }

    pub fn start_with(env: &Env, opts: Options) -> Server {
        let port = free_port();
        let tmp = env.root.join(format!("tmp-{port}"));
        std::fs::create_dir_all(&tmp).unwrap();
        let config = env.root.join(format!("server-config-{port}.toml"));
        let mut vars = Server::env_for(env, port, &tmp, &config);
        for (k, v) in &opts.extra_env {
            vars.retain(|(n, _)| n != k);
            if !v.is_empty() {
                vars.push((k.clone(), v.clone()));
            }
        }
        Server::spawn(env, port, tmp, config, vars, &opts.args, opts.no_wait)
    }

    /// Start again, with the same folders and an empty data folder (a restart on a new host).
    pub fn restart_empty(self, env: &Env) -> Server {
        let port = self.port;
        let config = self.config.clone();
        drop(self);
        let tmp = env.root.join(format!("tmp-{port}-b"));
        std::fs::create_dir_all(&tmp).unwrap();
        let vars = Server::env_for(env, port, &tmp, &config);
        Server::spawn(env, port, tmp, config, vars, &[], false)
    }

    fn spawn(
        _env: &Env,
        port: u16,
        tmp: PathBuf,
        config: PathBuf,
        vars: Vec<(String, String)>,
        args: &[String],
        no_wait: bool,
    ) -> Server {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_waymark"));
        cmd.args(args)
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .env("HOME", tmp.join("home"))
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in &vars {
            cmd.env(k, v);
        }
        std::fs::create_dir_all(tmp.join("home")).unwrap();
        let mut child = cmd.spawn().expect("start waymark");
        let out = Arc::new(Mutex::new(Vec::new()));
        for stream in [
            Box::new(child.stdout.take().unwrap()) as Box<dyn std::io::Read + Send>,
            Box::new(child.stderr.take().unwrap()),
        ] {
            let sink = out.clone();
            std::thread::spawn(move || {
                for line in BufReader::new(stream).lines().map_while(Result::ok) {
                    sink.lock().unwrap().push(line);
                }
            });
        }
        let server = Server {
            child,
            port,
            base: format!("http://127.0.0.1:{port}"),
            tmp,
            config,
            out,
        };
        if !no_wait {
            server.wait_ready();
        }
        server
    }

    fn wait_ready(&self) {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(20) {
            if http_get(&format!("{}/health", self.base)).status == 200 {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("the server did not start. Output: {:?}", self.output());
    }

    pub fn output(&self) -> Vec<String> {
        self.out.lock().unwrap().clone()
    }

    /// Wait until the process ends. Returns the exit code and all output.
    pub fn wait_exit(&mut self) -> (Option<i32>, String) {
        let start = Instant::now();
        loop {
            if let Some(s) = self.child.try_wait().unwrap() {
                std::thread::sleep(Duration::from_millis(200));
                return (s.code(), self.output().join("\n"));
            }
            if start.elapsed() > Duration::from_secs(15) {
                panic!("the server did not stop");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ---- HTTP helpers --------------------------------------------------------

pub struct Resp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Resp {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or(Value::Null)
    }
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .timeout_global(Some(Duration::from_secs(120)))
        .build()
        .into()
}

fn to_resp(mut r: ureq::http::Response<ureq::Body>) -> Resp {
    Resp {
        status: r.status().as_u16(),
        headers: r
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect(),
        body: r.body_mut().read_to_string().unwrap_or_default(),
    }
}

pub fn http_get(url: &str) -> Resp {
    match agent().get(url).call() {
        Ok(r) => to_resp(r),
        Err(_) => Resp {
            status: 0,
            headers: vec![],
            body: String::new(),
        },
    }
}

pub fn http_post_form(url: &str, form: &[(&str, &str)]) -> Resp {
    to_resp(agent().post(url).send_form(form.iter().copied()).unwrap())
}

pub fn http_post_json(
    url: &str,
    body: &Value,
    bearer: Option<&str>,
    extra: &[(&str, &str)],
) -> Resp {
    let mut req = agent()
        .post(url)
        .header("Accept", "application/json, text/event-stream");
    if let Some(t) = bearer {
        req = req.header("Authorization", &format!("Bearer {t}"));
    }
    for (k, v) in extra {
        req = req.header(*k, *v);
    }
    to_resp(req.send_json(body).unwrap())
}

// ---- The sign-in flow of a client ---------------------------------------

pub fn pkce() -> (String, String) {
    let verifier = URL_SAFE_NO_PAD.encode(uuid::Uuid::new_v4().as_bytes())
        + &URL_SAFE_NO_PAD.encode(uuid::Uuid::new_v4().as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

pub fn query_param(url: &str, key: &str) -> Option<String> {
    let q = url.split_once('?')?.1;
    q.split('&').find_map(|p| {
        let (k, v) = p.split_once('=')?;
        (k == key).then(|| percent_decode(v))
    })
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(if b[i] == b'+' { b' ' } else { b[i] });
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

pub fn encode(s: &str) -> String {
    waymark::auth::urlencode(s)
}

pub struct Tokens {
    pub access: String,
    pub refresh: String,
    pub client_id: String,
}

/// Register a client automatically. Returns the client ID.
pub fn register_client(server: &Server, redirect: &str) -> String {
    let r = http_post_json(
        &server.url("/register"),
        &json!({"redirect_uris": [redirect], "client_name": "Test client"}),
        None,
        &[],
    );
    assert_eq!(r.status, 201, "{}", r.body);
    r.json()["client_id"].as_str().unwrap().to_string()
}

/// Run the whole sign-in as a browser would, with the account behind `code`. Returns the
/// response of the last step before the token request: the redirect to the client, or the
/// refusal page of the server.
pub fn browse(
    server: &Server,
    client_id: &str,
    redirect: &str,
    challenge: &str,
    code: &str,
) -> Resp {
    let authorize = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&state=xyz&code_challenge={}&code_challenge_method=S256&resource={}",
        server.url("/authorize"),
        encode(client_id),
        encode(redirect),
        challenge,
        encode(&server.url("/mcp"))
    );
    let r = http_get(&authorize);
    assert_eq!(r.status, 302, "authorize: {} {}", r.status, r.body);
    let provider = r.header("location").unwrap().to_string();
    let state = query_param(&provider, "state").expect("state");
    http_get(&format!(
        "{}?code={}&state={}",
        server.url("/callback"),
        encode(code),
        encode(&state)
    ))
}

/// The full flow for the owner, with an automatically registered client.
pub fn sign_in(server: &Server, env: &Env, code: &str) -> Tokens {
    let redirect = "http://localhost:7777/cb";
    env.fake.set_account(code, Env::owner_account());
    let client_id = register_client(server, redirect);
    sign_in_with(server, &client_id, redirect, code)
}

pub fn sign_in_with(server: &Server, client_id: &str, redirect: &str, code: &str) -> Tokens {
    let (verifier, challenge) = pkce();
    let r = browse(server, client_id, redirect, &challenge, code);
    assert_eq!(r.status, 302, "callback: {} {}", r.status, r.body);
    let back = r.header("location").unwrap().to_string();
    assert!(back.starts_with(redirect), "{back}");
    let auth_code = query_param(&back, "code").expect("code");
    let t = http_post_form(
        &server.url("/token"),
        &[
            ("grant_type", "authorization_code"),
            ("code", &auth_code),
            ("redirect_uri", redirect),
            ("client_id", client_id),
            ("code_verifier", &verifier),
            ("resource", &server.url("/mcp")),
        ],
    );
    assert_eq!(t.status, 200, "token: {}", t.body);
    let j = t.json();
    Tokens {
        access: j["access_token"].as_str().unwrap().to_string(),
        refresh: j["refresh_token"].as_str().unwrap().to_string(),
        client_id: client_id.to_string(),
    }
}

// ---- MCP over HTTP -------------------------------------------------------

/// Read the JSON-RPC answer from a JSON or event-stream body.
fn rpc_body(r: &Resp) -> Value {
    if let Ok(v) = serde_json::from_str::<Value>(&r.body) {
        return v;
    }
    for line in r.body.lines() {
        if let Some(d) = line.strip_prefix("data:") {
            if let Ok(v) = serde_json::from_str::<Value>(d.trim()) {
                if v.get("result").is_some() || v.get("error").is_some() {
                    return v;
                }
            }
        }
    }
    Value::Null
}

pub fn mcp_post(server: &Server, token: Option<&str>, body: &Value) -> Resp {
    http_post_json(
        &server.url("/mcp"),
        body,
        token,
        &[("Content-Type", "application/json")],
    )
}

pub fn mcp_initialize(server: &Server, token: &str) -> Value {
    let r = mcp_post(
        server,
        Some(token),
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion":"2025-06-18","capabilities":{},
            "clientInfo":{"name":"test","version":"1"}}}),
    );
    rpc_body(&r)
}

/// Call a tool. Returns the structured content and whether it is an error.
pub fn call_tool(server: &Server, token: &str, name: &str, args: Value) -> (Value, bool) {
    let r = mcp_post(
        server,
        Some(token),
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":name,"arguments":args}}),
    );
    let v = rpc_body(&r);
    let result = &v["result"];
    let structured = result
        .get("structuredContent")
        .cloned()
        .unwrap_or_else(|| result.clone());
    (structured, result["isError"].as_bool().unwrap_or(false))
}

pub fn list_tools(server: &Server, token: &str) -> Vec<String> {
    let r = mcp_post(
        server,
        Some(token),
        &json!({"jsonrpc":"2.0","id":3,"method":"tools/list","params":{}}),
    );
    rpc_body(&r)["result"]["tools"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|t| t["name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
