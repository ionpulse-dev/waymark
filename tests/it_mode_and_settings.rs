//! The modes and the settings of the server (see `specs/foundation/personal-server/test.md`,
//! A21 to A26). A start that must fail is checked here, with the real binary.

mod common;
use common::server::*;
use common::{McpClient, Spawn};

/// Start with a change of the environment. The start must fail. Returns the exit code and output.
fn failing_start(change: &[(&str, &str)], args: &[&str]) -> (Option<i32>, String) {
    let env = Env::new(false);
    let mut server = Server::start_with(
        &env,
        Options {
            extra_env: change
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            args: args.iter().map(|s| s.to_string()).collect(),
            no_wait: true,
        },
    );
    server.wait_exit()
}

// A21
#[test]
fn an_unknown_mode_stops_the_start_and_lists_the_modes() {
    let (code, out) = failing_start(&[("WAYMARK_MODE", "remote")], &[]);
    assert_eq!(code, Some(2), "{out}");
    for m in ["local", "personal-server", "team-server"] {
        assert!(out.contains(m), "{out}");
    }
    // The same through the argument.
    let (code, out) = failing_start(&[("WAYMARK_MODE", "")], &["--mode", "nonsense"]);
    assert_eq!(code, Some(2), "{out}");
    assert!(out.contains("personal-server"), "{out}");
}

// A22
#[test]
fn team_server_is_not_available_yet() {
    let (code, out) = failing_start(&[("WAYMARK_MODE", "team-server")], &[]);
    assert_eq!(code, Some(2), "{out}");
    assert!(out.contains("not available yet"), "{out}");
}

// A23
#[test]
fn a_public_url_without_https_stops_the_start() {
    let (code, out) = failing_start(&[("WAYMARK_PUBLIC_URL", "http://mem.example.org")], &[]);
    assert_eq!(code, Some(2), "{out}");
    assert!(out.contains("https://"), "{out}");
}

// A24
#[test]
fn a_missing_setting_stops_the_start_and_is_named() {
    let (code, out) = failing_start(&[("WAYMARK_OWNER", "")], &[]);
    assert_eq!(code, Some(2), "{out}");
    assert!(out.contains("WAYMARK_OWNER"), "{out}");
    let (code, out) = failing_start(&[("WAYMARK_GITHUB_APP_KEY", "")], &[]);
    assert_eq!(code, Some(2), "{out}");
    assert!(out.contains("WAYMARK_GITHUB_APP_KEY"), "{out}");
}

#[test]
fn settings_of_another_mode_stop_the_start() {
    let (code, out) = failing_start(&[("WAYMARK_ROLES_FILE", "/etc/roles.toml")], &[]);
    assert_eq!(code, Some(2), "{out}");
    assert!(out.contains("WAYMARK_ROLES_FILE"), "{out}");
    let (code, out) = failing_start(&[], &["--author-email", "x@example.org"]);
    assert_eq!(code, Some(2), "{out}");
    assert!(out.contains("--author-email"), "{out}");
}

#[test]
fn a_wrong_private_key_stops_the_start_and_never_prints_it() {
    let (code, out) = failing_start(&[("WAYMARK_GITHUB_APP_KEY", "not a key SECRETVALUE")], &[]);
    assert_eq!(code, Some(2), "{out}");
    assert!(out.contains("private key"), "{out}");
    assert!(!out.contains("SECRETVALUE"), "{out}");
}

// A25
#[test]
fn no_mode_runs_local_mode_over_stdio() {
    let world = common::World::new();
    let home = world.root.join("home-x");
    let mut client = McpClient::start(
        &home,
        Spawn {
            args: vec![
                "--path".into(),
                world.a.display().to_string(),
                "--author-name".into(),
                "Anna Example".into(),
                "--author-email".into(),
                "anna@example.org".into(),
            ],
            ..Spawn::default()
        },
    );
    let names = client.tool_names();
    assert!(names.contains(&"status".to_string()), "{names:?}");
    assert!(client.close());
}

// A26
#[test]
fn the_server_listens_on_the_given_port() {
    let env = Env::new(false);
    let server = Server::start(&env);
    // `Server::start` sets PORT, and waits for the health answer on that port.
    assert_eq!(http_get(&server.url("/health")).status, 200);
}

#[test]
fn print_client_gives_the_manual_client_of_the_signing_key() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_waymark"))
        .arg("--print-client")
        .env_clear()
        .env("WAYMARK_SIGNING_KEY", SIGNING_KEY)
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let (id, secret) = waymark::auth::tokens::manual_client(SIGNING_KEY);
    assert!(text.contains(&format!("Client ID: {id}")), "{text}");
    assert!(text.contains(&format!("Client secret: {secret}")), "{text}");
    // Without a key it stops and prints no secret.
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_waymark"))
        .arg("--print-client")
        .env_clear()
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}
