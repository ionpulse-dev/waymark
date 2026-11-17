//! End-to-end tests of the personal server (see `specs/foundation/personal-server/test.md`).
//! The real binary runs in `--mode personal-server` against a fake GitHub. No vendor is needed.

mod common;
use common::fake_github::Account;
use common::git::git;
use common::server::*;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

/// Call `status` until the memory repository is ready.
fn wait_ready(server: &Server, token: &str) {
    let start = Instant::now();
    loop {
        let (v, _) = call_tool(server, token, "status", json!({}));
        if v["memory_repository"]["state"] == "ready" || v["memory_repository"]["state"] == "empty"
        {
            return;
        }
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "the clone did not finish: {v}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn add_doc(server: &Server, token: &str, title: &str, body: &str) -> (Value, bool) {
    call_tool(
        server,
        token,
        "add_document",
        json!({
            "folder": "notes",
            "title": title,
            "description": format!("A test document about {title}."),
            "body": body,
        }),
    )
}

fn bare(env: &Env) -> std::path::PathBuf {
    env.fake.git_root.join("memory.git")
}

// A1
#[test]
fn a_request_with_no_token_gets_401_and_the_discovery_information() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let r = mcp_post(
        &server,
        None,
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
    );
    assert_eq!(r.status, 401);
    let challenge = r.header("www-authenticate").unwrap();
    assert!(challenge.starts_with("Bearer "), "{challenge}");
    assert!(
        challenge.contains(&server.url("/.well-known/oauth-protected-resource/mcp")),
        "{challenge}"
    );
    for path in [
        "/.well-known/oauth-protected-resource",
        "/.well-known/oauth-protected-resource/mcp",
    ] {
        let m = http_get(&server.url(path)).json();
        assert_eq!(m["resource"], server.url("/mcp"));
        assert_eq!(m["authorization_servers"][0], server.base);
    }
    let a = http_get(&server.url("/.well-known/oauth-authorization-server")).json();
    assert_eq!(a["issuer"], server.base);
    assert_eq!(a["code_challenge_methods_supported"][0], "S256");
    assert_eq!(a["client_id_metadata_document_supported"], true);
    assert!(a["token_endpoint"].as_str().unwrap().ends_with("/token"));
}

// A2
#[test]
fn a_client_document_is_read_and_checked() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let url = format!("{}/client/doc.json", env.fake.base);
    let redirect = "http://localhost:7777/cb";
    env.fake.set_account("owner-code", Env::owner_account());
    env.fake.set_client_doc(json!({
        "client_id": url, "client_name": "Doc client", "redirect_uris": [redirect]
    }));
    let t = sign_in_with(&server, &url, redirect, "owner-code");
    assert!(!t.access.is_empty());

    // A document whose client_id differs from its address is refused.
    env.fake.set_client_doc(json!({
        "client_id": "https://other.example/doc.json", "client_name": "X", "redirect_uris": [redirect]
    }));
    let (_, challenge) = pkce();
    let r = http_get(&format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256",
        server.url("/authorize"), encode(&url), encode(redirect), challenge
    ));
    assert_eq!(r.status, 400);

    // A redirect address that the document does not name is refused.
    env.fake.set_client_doc(json!({
        "client_id": url, "client_name": "Doc client", "redirect_uris": [redirect]
    }));
    let r = http_get(&format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256",
        server.url("/authorize"), encode(&url), encode("http://localhost:9999/evil"), challenge
    ));
    assert_eq!(r.status, 400);
}

// A3
#[test]
fn automatic_registration_works_and_needs_no_stored_state() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let redirect = "http://localhost:7777/cb";
    let client = register_client(&server, redirect);
    // A redirect address that is not https or local is refused at registration.
    let bad = http_post_json(
        &server.url("/register"),
        &json!({"redirect_uris": ["http://evil.example/cb"]}),
        None,
        &[],
    );
    assert_eq!(bad.status, 400);
    // A restarted server (same key) knows the client: it is a signed value.
    let server = server.restart_empty(&env);
    env.fake.set_account("c1", Env::owner_account());
    let t = sign_in_with(&server, &client, redirect, "c1");
    assert!(!t.access.is_empty());
}

// A4
#[test]
fn the_manual_client_needs_its_secret() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let (id, secret) = waymark::auth::tokens::manual_client(SIGNING_KEY);
    let redirect = "https://chat.example/callback";
    env.fake.set_account("c1", Env::owner_account());
    let (verifier, challenge) = pkce();
    let r = browse(&server, &id, redirect, &challenge, "c1");
    assert_eq!(r.status, 302);
    let code = query_param(r.header("location").unwrap(), "code").unwrap();
    let form = |secret: &str| {
        http_post_form(
            &server.url("/token"),
            &[
                ("grant_type", "authorization_code"),
                ("code", &code),
                ("redirect_uri", redirect),
                ("client_id", &id),
                ("client_secret", secret),
                ("code_verifier", &verifier),
            ],
        )
    };
    assert_eq!(form("wrong").status, 401);
    let ok = form(&secret);
    assert_eq!(ok.status, 200, "{}", ok.body);
}

// A5
#[test]
fn the_owner_gets_a_token_for_this_server_and_never_a_github_token() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let t = sign_in(&server, &env, "owner-code");
    assert!(!t.access.contains("gho_") && !t.refresh.contains("gho_"));
    // The token works on this server.
    assert!(list_tools(&server, &t.access).contains(&"status".to_string()));
    // A wrong PKCE verifier is refused.
    let redirect = "http://localhost:7777/cb";
    let (_, challenge) = pkce();
    let r = browse(&server, &t.client_id, redirect, &challenge, "owner-code");
    let code = query_param(r.header("location").unwrap(), "code").unwrap();
    let bad = http_post_form(
        &server.url("/token"),
        &[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", redirect),
            ("client_id", &t.client_id),
            ("code_verifier", &"a".repeat(50)),
        ],
    );
    assert_eq!(bad.status, 400);
    assert_eq!(bad.json()["error"], "invalid_grant");
    // A resource that is not this server is refused at the door.
    let (_, challenge) = pkce();
    let r = http_get(&format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256&resource={}",
        server.url("/authorize"), encode(&t.client_id), encode(redirect), challenge,
        encode("https://other.example/mcp")
    ));
    assert_eq!(r.status, 400);
    // The token response holds no GitHub value at all.
    let (verifier, challenge) = pkce();
    let r = browse(&server, &t.client_id, redirect, &challenge, "owner-code");
    let code = query_param(r.header("location").unwrap(), "code").unwrap();
    let ok = http_post_form(
        &server.url("/token"),
        &[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", redirect),
            ("client_id", &t.client_id),
            ("code_verifier", &verifier),
        ],
    );
    assert!(!ok.body.contains("gho_"), "{}", ok.body);
}

// A6
#[test]
fn another_account_is_refused_and_gets_no_token() {
    let env = Env::new(false);
    let server = Server::start(&env);
    env.fake.set_account(
        "stranger",
        Account {
            login: "stranger".into(),
            name: None,
            emails: vec![("s@example.org".into(), true, true)],
        },
    );
    let redirect = "http://localhost:7777/cb";
    let client = register_client(&server, redirect);
    let (_, challenge) = pkce();
    let r = browse(&server, &client, redirect, &challenge, "stranger");
    assert_eq!(r.status, 403);
    assert!(r.header("location").is_none());
    assert!(!r.body.contains("code="));
    assert!(server
        .output()
        .iter()
        .any(|l| l.contains("account=\"stranger\"") && l.contains("refused")));
}

// A7
#[test]
fn a_token_survives_a_restart_with_an_empty_data_folder() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let t = sign_in(&server, &env, "owner-code");
    wait_ready(&server, &t.access);
    let old_tmp = server.tmp.clone();
    let server = server.restart_empty(&env);
    assert_ne!(server.tmp, old_tmp);
    // The same token works, and the first call clones again.
    wait_ready(&server, &t.access);
    let (v, err) = call_tool(&server, &t.access, "search", json!({"query": "anything"}));
    assert!(!err, "{v}");
}

// A8
#[test]
fn an_expired_token_is_refused_and_the_renewal_token_renews_it() {
    let env = Env::new(false);
    let server = Server::start_with(
        &env,
        Options {
            extra_env: vec![("WAYMARK_ACCESS_TOKEN_SECONDS".into(), "2".into())],
            ..Options::default()
        },
    );
    let t = sign_in(&server, &env, "owner-code");
    assert_eq!(
        mcp_post(
            &server,
            Some(&t.access),
            &json!({"jsonrpc":"2.0","id":1,"method":"tools/list"})
        )
        .status,
        200
    );
    std::thread::sleep(Duration::from_secs(3));
    let r = mcp_post(
        &server,
        Some(&t.access),
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
    );
    assert_eq!(r.status, 401);
    assert!(r
        .header("www-authenticate")
        .unwrap()
        .contains("invalid_token"));
    // Renew.
    let renewed = http_post_form(
        &server.url("/token"),
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", &t.refresh),
            ("client_id", &t.client_id),
        ],
    );
    assert_eq!(renewed.status, 200, "{}", renewed.body);
    let new_access = renewed.json()["access_token"].as_str().unwrap().to_string();
    assert!(list_tools(&server, &new_access).contains(&"status".to_string()));
    // A renewal token of another client is refused.
    let other = register_client(&server, "http://localhost:8888/cb");
    let bad = http_post_form(
        &server.url("/token"),
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", &t.refresh),
            ("client_id", &other),
        ],
    );
    assert_eq!(bad.status, 400);
    // An access token is not a renewal token.
    let bad = http_post_form(
        &server.url("/token"),
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", &t.access),
            ("client_id", &t.client_id),
        ],
    );
    assert_eq!(bad.status, 400);
}

// A9
#[test]
fn a_changed_signing_key_refuses_old_tokens() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let t = sign_in(&server, &env, "owner-code");
    drop(server);
    let server = Server::start_with(
        &env,
        Options {
            extra_env: vec![(
                "WAYMARK_SIGNING_KEY".into(),
                "another-signing-key-another-signing-key".into(),
            )],
            ..Options::default()
        },
    );
    let r = mcp_post(
        &server,
        Some(&t.access),
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
    );
    assert_eq!(r.status, 401);
}

// A10, A28
#[test]
fn a_write_is_on_the_remote_with_the_signed_in_person_as_author() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let t = sign_in(&server, &env, "owner-code");
    wait_ready(&server, &t.access);
    let (v, err) = add_doc(
        &server,
        &t.access,
        "First note",
        "Hello from the personal server.",
    );
    assert!(!err, "{v}");
    let log = git(&bare(&env), &["log", "--format=%an <%ae>|%s", "-n", "3"]);
    assert!(log.contains("Olivia Owner <olivia@example.org>|"), "{log}");
    let files = git(&bare(&env), &["ls-tree", "-r", "--name-only", "main"]);
    let id = v["id"].as_str().unwrap();
    assert!(files.contains(&format!("{id}.md")), "{files}");
}

// A11
#[test]
fn a_server_limit_makes_the_access_read_only() {
    let env = Env::new(false);
    let mut server = Server::start(&env);
    std::fs::write(&server.config, "[limits]\naccess = \"read-only\"\n").unwrap();
    let t = sign_in(&server, &env, "owner-code");
    wait_ready(&server, &t.access);
    let tools = list_tools(&server, &t.access);
    assert!(!tools.contains(&"add_document".to_string()), "{tools:?}");
    let (v, err) = add_doc(&server, &t.access, "Blocked", "No.");
    assert!(err, "{v}");
    let (_, err) = call_tool(&server, &t.access, "search", json!({"query": "x"}));
    assert!(!err);
    let _ = server.child.try_wait();
}

// A12
#[test]
fn a_stopped_and_started_process_serves_with_no_data_from_before() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let t = sign_in(&server, &env, "owner-code");
    wait_ready(&server, &t.access);
    let (added, err) = add_doc(&server, &t.access, "Kept note", "Stored in Git.");
    assert!(!err, "{added}");
    let server = server.restart_empty(&env);
    wait_ready(&server, &t.access);
    let (v, err) = call_tool(&server, &t.access, "search", json!({"query": "Kept note"}));
    assert!(!err, "{v}");
    assert!(v.to_string().contains(added["id"].as_str().unwrap()), "{v}");
}

// A14, A15, A17
#[test]
fn the_status_page_is_static_and_shows_nothing_private() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let r = http_get(&server.url("/"));
    assert_eq!(r.status, 200);
    assert!(r.header("content-type").unwrap().starts_with("text/html"));
    assert_eq!(
        r.header("content-security-policy"),
        Some("default-src 'none'; style-src 'unsafe-inline'")
    );
    assert_eq!(r.header("x-content-type-options"), Some("nosniff"));
    assert!(r.header("set-cookie").is_none());
    let b = &r.body;
    assert!(b.contains(">Waymark<") && b.contains("The server is running."));
    assert!(b.contains("https://github.com/ionpulse-dev/waymark"));
    assert!(b.contains("#1d4ed8") && b.contains("color:#fff"));
    assert!(b.contains("name=\"viewport\""));
    // No script, no external file, no private fact.
    for forbidden in [
        "<script", "src=", "@import", "url(", "<link", "<img", "<iframe",
    ] {
        assert!(!b.contains(forbidden), "page holds {forbidden}");
    }
    let links: Vec<&str> = b.match_indices("http").map(|(i, _)| &b[i..]).collect();
    assert_eq!(links.len(), 1, "{links:?}");
    for private in ["owner", "memory.git", "0.5.0", "clone", "version"] {
        assert!(!b.to_lowercase().contains(private), "page holds {private}");
    }
    // The text is centered and wraps, so a narrow screen does not scroll sideways.
    assert!(b.contains("text-align:center") && b.contains("overflow-wrap:anywhere"));
    assert!(!b.contains("width:") || !b.contains("px"));
    // White on #1d4ed8 has a contrast ratio of at least 4.5.
    let lum = |c: u8| {
        let c = c as f64 / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let blue = 0.2126 * lum(0x1d) + 0.7152 * lum(0x4e) + 0.0722 * lum(0xd8);
    assert!(1.05 / (blue + 0.05) >= 4.5);
    let h = http_get(&server.url("/health"));
    assert_eq!((h.status, h.body.as_str()), (200, "ok"));
}

// A16
#[test]
fn the_status_page_and_health_answer_at_once_while_the_clone_runs() {
    let env = Env::new(false);
    env.fake.set_git_delay_ms(4000);
    let server = Server::start(&env);
    let start = Instant::now();
    assert_eq!(http_get(&server.url("/")).status, 200);
    assert_eq!(http_get(&server.url("/health")).status, 200);
    assert!(start.elapsed() < Duration::from_secs(2));
}

// A18, A19
#[test]
fn the_log_has_one_line_per_call_and_no_secret() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let t = sign_in(&server, &env, "owner-code");
    wait_ready(&server, &t.access);
    let (added, _) = add_doc(
        &server,
        &t.access,
        "Secret test",
        &format!("Body with {DOC_MARKER} inside."),
    );
    let doc_id = added["id"].as_str().unwrap().to_string();
    call_tool(&server, &t.access, "show", json!({"id": doc_id}));
    call_tool(&server, &t.access, "show", json!({"id": "notes/missing"}));
    let out = server.output().join("\n");
    for secret in [
        SIGNING_KEY,
        APP_SECRET_MARKER,
        DOC_MARKER,
        t.access.as_str(),
        t.refresh.as_str(),
        "PRIVATE KEY",
        "gho_",
        "ghs_fake",
    ] {
        assert!(!out.contains(secret), "the log holds {secret}");
    }
    let calls: Vec<&str> = out
        .lines()
        .filter(|l| l.contains("event=tool_call"))
        .collect();
    let shows: Vec<&&str> = calls
        .iter()
        .filter(|l| l.contains("tool=\"show\""))
        .collect();
    assert_eq!(shows.len(), 2, "{out}");
    assert!(
        shows[0].contains("outcome=\"ok\"") && shows[0].contains(&format!("document=\"{doc_id}\""))
    );
    assert!(
        shows[1].contains("outcome=\"error\"") && shows[1].contains("document=\"notes/missing\"")
    );
    assert!(calls.iter().all(|l| l.starts_with("time=")));
    assert!(out.contains("event=start") && out.contains("event=sign_in"));
}

// A20
#[test]
fn init_on_an_empty_repository_works_in_the_same_session() {
    let env = Env::new(true);
    let server = Server::start(&env);
    let t = sign_in(&server, &env, "owner-code");
    wait_ready(&server, &t.access);
    let (v, _) = call_tool(&server, &t.access, "status", json!({}));
    assert_eq!(v["memory_repository"]["state"], "empty", "{v}");
    let (v, err) = call_tool(
        &server,
        &t.access,
        "init",
        json!({"name": "Personal memory"}),
    );
    assert!(!err, "{v}");
    // The branch is whatever the clone of the empty repository had (main or master).
    let branch = git(
        &bare(&env),
        &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
    );
    let branch = branch
        .lines()
        .next()
        .expect("a branch on the remote")
        .to_string();
    let tree = git(&bare(&env), &["ls-tree", "-r", "--name-only", &branch]);
    assert!(tree.contains(".waymark/settings.toml"), "{tree}");
    let (v, err) = add_doc(&server, &t.access, "After init", "Works.");
    assert!(!err, "{v}");
}

// A27
#[test]
fn path_is_ignored_and_the_log_says_so() {
    let env = Env::new(false);
    let server = Server::start_with(
        &env,
        Options {
            args: vec!["--path".into(), "/nonexistent/place".into()],
            ..Options::default()
        },
    );
    let t = sign_in(&server, &env, "owner-code");
    wait_ready(&server, &t.access);
    assert!(server
        .output()
        .iter()
        .any(|l| l.contains("option_ignored") && l.contains("--path")));
    assert!(server.tmp.join("waymark-clone").exists());
}

// A29
#[test]
fn an_account_with_no_usable_email_is_refused() {
    let env = Env::new(false);
    let server = Server::start(&env);
    env.fake.set_account(
        "noemail",
        Account {
            login: "owner".into(),
            name: None,
            emails: vec![
                ("unverified@example.org".into(), false, true),
                ("1+owner@users.noreply.github.com".into(), true, false),
            ],
        },
    );
    let redirect = "http://localhost:7777/cb";
    let client = register_client(&server, redirect);
    let (_, challenge) = pkce();
    let r = browse(&server, &client, redirect, &challenge, "noemail");
    assert_eq!(r.status, 403);
    assert!(
        r.body
            .contains("Add and verify an email address on your GitHub account."),
        "{}",
        r.body
    );
    assert!(r.header("location").is_none());
}

// A30
#[test]
fn the_email_domain_list_of_the_memory_repository_is_applied() {
    let env = Env::new(false);
    // The memory repository allows only one domain.
    let seed = env.root.join("seed");
    let settings = seed.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&settings).unwrap();
    std::fs::write(
        &settings,
        format!("{text}\n[people]\nemail_domains = [\"corp.example\"]\n"),
    )
    .unwrap();
    git(&seed, &["add", "-A"]);
    git(
        &seed,
        &[
            "-c",
            "user.name=S",
            "-c",
            "user.email=s@example.org",
            "commit",
            "-q",
            "-m",
            "Limit domains",
        ],
    );
    git(&seed, &["push", "-q"]);
    let server = Server::start(&env);
    // The first sign-in uses an address in the allowed domain, so it passes whether or not the
    // clone is ready yet. On a fast machine the clone is ready before it.
    let redirect = "http://localhost:7777/cb";
    env.fake.set_account(
        "corp-code",
        Account {
            login: "owner".into(),
            name: Some("Olivia Owner".into()),
            emails: vec![("olivia@corp.example".into(), true, true)],
        },
    );
    let first = register_client(&server, redirect);
    let t0 = sign_in_with(&server, &first, redirect, "corp-code");
    // Wait for the clone: the domain list is read from it at the next sign-in.
    wait_ready(&server, &t0.access);
    env.fake.set_account("owner-code", Env::owner_account());
    let client = register_client(&server, redirect);
    let (_, challenge) = pkce();
    let r = browse(&server, &client, redirect, &challenge, "owner-code");
    assert_eq!(r.status, 403, "{}", r.body);
}

// A31
#[test]
fn a_call_waits_for_the_clone_and_then_says_to_call_again() {
    let env = Env::new(false);
    env.fake.set_git_delay_ms(6000);
    let server = Server::start_with(
        &env,
        Options {
            extra_env: vec![("WAYMARK_CLONE_WAIT_SECONDS".into(), "1".into())],
            ..Options::default()
        },
    );
    let t = sign_in(&server, &env, "owner-code");
    let (v, err) = call_tool(&server, &t.access, "search", json!({"query": "x"}));
    assert!(err, "{v}");
    assert!(v.to_string().contains("Call again"), "{v}");
    // After the clone, calls work.
    env.fake.set_git_delay_ms(0);
    let start = Instant::now();
    loop {
        let (v, err) = call_tool(&server, &t.access, "search", json!({"query": "x"}));
        if !err {
            break;
        }
        assert!(start.elapsed() < Duration::from_secs(30), "{v}");
        std::thread::sleep(Duration::from_millis(300));
    }
}

// A32
#[test]
fn the_tool_list_is_the_same_as_in_local_mode() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let t = sign_in(&server, &env, "owner-code");
    let remote = list_tools(&server, &t.access);
    let all = waymark::tools::registry().all_names();
    let mut a: Vec<String> = remote;
    a.sort();
    let mut b: Vec<String> = all.iter().map(|s| s.to_string()).collect();
    b.sort();
    // `approval` tools exist only when the memory repository uses verification.
    for name in &a {
        assert!(b.contains(name), "{name}");
    }
    assert!(a.len() >= 22, "{a:?}");
}

// A33
#[test]
fn a_refused_git_token_is_renewed_and_the_call_is_tried_again() {
    let env = Env::new(false);
    let server = Server::start(&env);
    let t = sign_in(&server, &env, "owner-code");
    wait_ready(&server, &t.access);
    let issued = env.fake.tokens_issued();
    assert!(issued >= 1);
    // The Git host stops accepting the token. The next write must make a new one.
    env.fake.expire_tokens();
    let (v, err) = add_doc(&server, &t.access, "After expiry", "Still works.");
    assert!(!err, "{v}");
    assert!(env.fake.git_refused() >= 1);
    assert!(env.fake.tokens_issued() > issued);
    let files = git(&bare(&env), &["ls-tree", "-r", "--name-only", "main"]);
    assert!(
        files.contains(&format!("{}.md", v["id"].as_str().unwrap())),
        "{files}"
    );
}

// A10 (the retry)
#[test]
fn a_rejected_push_is_retried_once_and_a_second_rejection_is_reported() {
    use std::os::unix::fs::PermissionsExt;
    let env = Env::new(false);
    let server = Server::start(&env);
    let t = sign_in(&server, &env, "owner-code");
    wait_ready(&server, &t.access);
    // A hook on the remote rejects the first push only.
    let hook = bare(&env).join("hooks/pre-receive");
    let marker = env.root.join("rejected-once");
    std::fs::write(
        &hook,
        format!(
            "#!/bin/sh\nif [ ! -e '{m}' ]; then touch '{m}'; echo 'temporary rejection' >&2; exit 1; fi\nexit 0\n",
            m = marker.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    let (v, err) = add_doc(
        &server,
        &t.access,
        "Retried note",
        "Pushed on the second try.",
    );
    assert!(!err, "{v}");
    assert!(marker.exists());
    let files = git(&bare(&env), &["ls-tree", "-r", "--name-only", "main"]);
    assert!(
        files.contains(&format!("{}.md", v["id"].as_str().unwrap())),
        "{files}"
    );
    // A hook that always rejects: the tool reports what happened.
    std::fs::write(&hook, "#!/bin/sh\necho 'always rejected' >&2\nexit 1\n").unwrap();
    let (v, err) = add_doc(&server, &t.access, "Rejected note", "Never pushed.");
    assert!(err, "{v}");
    assert!(v.to_string().contains("push was rejected"), "{v}");
}
