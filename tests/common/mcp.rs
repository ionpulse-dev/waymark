//! A small MCP client for tests: it starts the real `waymark` binary and speaks
//! newline-delimited JSON-RPC over stdio.

use super::world::{Person, World};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(30);

/// The answer to a request from the server (for example a question to the person).
pub type Hook = Box<dyn FnMut(&Value) -> Value + Send>;

/// The result of a tool call.
#[derive(Debug, Clone)]
pub struct Reply {
    pub is_error: bool,
    /// The structured content: `message`, `settings`, `notices`, and the tool's fields.
    pub json: Value,
}

impl Reply {
    pub fn code(&self) -> Option<&str> {
        self.json["code"].as_str()
    }
    pub fn message(&self) -> &str {
        self.json["message"].as_str().unwrap_or("")
    }
}

pub struct McpClient {
    child: Child,
    stdin: Arc<Mutex<Option<std::process::ChildStdin>>>,
    rx: Receiver<Value>,
    next_id: i64,
    /// The personal home of this client: `HOME`, with `.config/waymark/config.toml` for personal limits.
    pub home: PathBuf,
    pub system_config: PathBuf,
    pub initialize_result: Value,
    /// Notifications from the server, in the order they came.
    notes: Arc<Mutex<Vec<Value>>>,
}

#[derive(Default)]
pub struct Spawn {
    pub args: Vec<String>,
    pub hook: Option<Hook>,
    /// Extra environment variables.
    pub env: Vec<(String, String)>,
    /// Ask for elicitation in `initialize`.
    pub elicitation: bool,
    /// The exact `elicitation` capability to announce (instead of `{}`).
    pub elicitation_caps: Option<Value>,
}

impl Spawn {
    pub fn args(args: &[&str]) -> Spawn {
        Spawn {
            args: args.iter().map(|s| s.to_string()).collect(),
            ..Spawn::default()
        }
    }
}

impl McpClient {
    /// Start the server for a clone of the world, as one person.
    pub fn for_person(world: &World, p: Person, mut spawn: Spawn) -> McpClient {
        let author = World::author(p);
        let mut args = vec![
            "--path".to_string(),
            world.dir(p).display().to_string(),
            "--author-name".into(),
            author.name,
            "--author-email".into(),
            author.email,
        ];
        args.append(&mut spawn.args);
        spawn.args = args;
        let home = world.root.join(match p {
            Person::A => "home-a",
            Person::B => "home-b",
        });
        McpClient::start(&home, spawn)
    }

    /// Start the server with a home folder. Initializes the session.
    pub fn start(home: &std::path::Path, spawn: Spawn) -> McpClient {
        std::fs::create_dir_all(home.join(".config")).unwrap();
        let system_config = home.join("system-config.toml");
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_waymark"));
        cmd.args(&spawn.args)
            .env("HOME", home)
            .env("XDG_CONFIG_HOME", home.join(".config"))
            .env("WAYMARK_SYSTEM_CONFIG", &system_config)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env_remove("WAYMARK_AUTHOR_NAME")
            .env_remove("WAYMARK_AUTHOR_EMAIL")
            .current_dir(home)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        for (k, v) in &spawn.env {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().expect("start waymark");
        let stdin = Arc::new(Mutex::new(child.stdin.take()));
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = channel::<Value>();
        let notes: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
        let notes_writer = notes.clone();
        let mut hook = spawn.hook;
        let writer = stdin.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                let Ok(msg) = serde_json::from_str::<Value>(&line) else {
                    continue;
                };
                // A request from the server: answer it with the hook.
                if msg.get("method").is_some() && msg.get("id").is_some() {
                    let result = match hook.as_mut() {
                        Some(h) => h(&msg),
                        None => json!({}),
                    };
                    // A hook result with the key `__error` answers with a JSON-RPC error.
                    let reply = if result.get("__error").is_some() {
                        json!({"jsonrpc": "2.0", "id": msg["id"], "error": {"code": -32603, "message": "The hook says no."}})
                    } else {
                        json!({"jsonrpc": "2.0", "id": msg["id"], "result": result})
                    };
                    if let Some(w) = writer.lock().unwrap().as_mut() {
                        let _ = writeln!(w, "{reply}");
                        let _ = w.flush();
                    }
                    continue;
                }
                if msg.get("method").is_some() && msg.get("id").is_none() {
                    notes_writer.lock().unwrap().push(msg.clone());
                }
                if tx.send(msg).is_err() {
                    break;
                }
            }
        });
        let mut c = McpClient {
            child,
            stdin,
            rx,
            next_id: 0,
            home: home.to_path_buf(),
            system_config,
            initialize_result: Value::Null,
            notes,
        };
        let caps = if let Some(e) = &spawn.elicitation_caps {
            json!({"elicitation": e})
        } else if spawn.elicitation {
            json!({"elicitation": {}})
        } else {
            json!({})
        };
        let init = c.request(
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": caps,
                "clientInfo": {"name": "waymark-tests", "version": "0"}
            }),
        );
        c.initialize_result = init["result"].clone();
        c.notify("notifications/initialized", json!({}));
        c
    }

    fn send(&self, v: &Value) {
        let mut guard = self.stdin.lock().unwrap();
        let w = guard.as_mut().expect("stdin is closed");
        writeln!(w, "{v}").unwrap();
        w.flush().unwrap();
    }

    pub fn notify(&self, method: &str, params: Value) {
        self.send(&json!({"jsonrpc": "2.0", "method": method, "params": params}));
    }

    /// Send a request and wait for its response (a JSON-RPC message with `result` or `error`).
    pub fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        loop {
            match self.rx.recv_timeout(TIMEOUT) {
                Ok(m) if m["id"] == json!(id) => return m,
                Ok(_) => continue,
                Err(RecvTimeoutError::Timeout) => panic!("No answer to {method} in {TIMEOUT:?}"),
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("waymark ended before it answered {method}")
                }
            }
        }
    }

    /// How many notifications with this method arrived so far. Call after a request, so that the
    /// reader thread has had time to read them.
    pub fn notifications(&self, method: &str) -> usize {
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.notes
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m["method"] == method)
            .count()
    }

    /// The params of the notifications with this method.
    pub fn notification_params(&self, method: &str) -> Vec<Value> {
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.notes
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m["method"] == method)
            .map(|m| m["params"].clone())
            .collect()
    }

    /// A tool call with a progress token.
    pub fn call_with_progress(&mut self, name: &str, args: Value) -> Reply {
        let r = self.request(
            "tools/call",
            json!({"name": name, "arguments": args, "_meta": {"progressToken": "p1"}}),
        );
        let res = &r["result"];
        Reply {
            is_error: res["isError"].as_bool().unwrap_or(false),
            json: res["structuredContent"].clone(),
        }
    }

    pub fn list_tools(&mut self) -> Vec<Value> {
        let r = self.request("tools/list", json!({}));
        r["result"]["tools"].as_array().cloned().unwrap_or_default()
    }

    pub fn tool_names(&mut self) -> Vec<String> {
        self.list_tools()
            .iter()
            .filter_map(|t| t["name"].as_str().map(String::from))
            .collect()
    }

    /// Call a tool. Panics on a protocol error: use `call_raw` to see one.
    pub fn call(&mut self, name: &str, args: Value) -> Reply {
        let r = self.call_raw(name, args);
        assert!(r.get("error").is_none(), "protocol error for {name}: {r}");
        let res = &r["result"];
        Reply {
            is_error: res["isError"].as_bool().unwrap_or(false),
            json: res["structuredContent"].clone(),
        }
    }

    pub fn call_raw(&mut self, name: &str, args: Value) -> Value {
        self.request("tools/call", json!({"name": name, "arguments": args}))
    }

    /// Close stdin and wait for the process to end. Returns true if it ended with success.
    pub fn close(mut self) -> bool {
        self.stdin.lock().unwrap().take();
        self.child.wait().map(|s| s.success()).unwrap_or(false)
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
