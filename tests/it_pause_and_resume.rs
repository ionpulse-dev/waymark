//! End-to-end tests of pause and resume (see `specs/features/pause-and-resume/test.md`).

mod common;
use common::git::{git, subjects};
use common::kb::commit_doc;
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::path::PathBuf;

fn personal_path(w: &World) -> PathBuf {
    w.root.join("home-a/.config/waymark/config.toml")
}

fn set(path: &std::path::Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn pause(w: &World, on: bool) {
    set(&personal_path(w), &format!("[limits]\npaused = {on}\n"));
}

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

fn add(c: &mut McpClient, title: &str) -> common::Reply {
    c.call(
        "add",
        json!({"folder": "ops", "title": title, "description": format!("About {title}."), "body": "x"}),
    )
}

#[test]
fn paused_blocks_writes() {
    let w = World::new();
    pause(&w, true);
    let mut c = client(&w);
    let before = (subjects(&w.a), subjects(&w.remote));
    let r = add(&mut c, "Doc one");
    assert_eq!(r.code(), Some("paused"));
    assert!(r.json["next"].as_str().is_some_and(|n| !n.is_empty()));
    assert_eq!((subjects(&w.a), subjects(&w.remote)), before);
}

#[test]
fn paused_allows_reads_and_status_names_pause() {
    let w = World::new();
    pause(&w, true);
    let mut c = client(&w);
    let s = c.call("status", json!({}));
    assert!(!s.is_error);
    assert_eq!(s.json["settings"]["paused"], true);
    assert_eq!(s.json["notices"][0], "Writes are paused.");
    let l = c.call("list", json!({}));
    assert!(!l.is_error, "{}", l.json);
    assert_eq!(l.json["notices"][0], "Writes are paused.");
}

#[test]
fn errors_while_paused_have_the_notice_too() {
    let w = World::new();
    pause(&w, true);
    let mut c = client(&w);
    let r = add(&mut c, "Doc one");
    assert_eq!(r.json["notices"][0], "Writes are paused.");
}

#[test]
fn lower_layer_cannot_resume() {
    let w = World::new();
    set(
        &w.root.join("home-a/system-config.toml"),
        "[limits]\npaused = true\n",
    );
    pause(&w, false);
    let mut c = client(&w);
    assert_eq!(add(&mut c, "Doc one").code(), Some("paused"));
}

#[test]
fn pause_has_no_timer() {
    let w = World::new();
    pause(&w, true);
    let two_days = std::time::SystemTime::now() - std::time::Duration::from_secs(2 * 86400);
    let f = std::fs::File::options()
        .write(true)
        .open(personal_path(&w))
        .unwrap();
    f.set_modified(two_days).unwrap();
    let mut c = client(&w);
    assert_eq!(add(&mut c, "Doc one").code(), Some("paused"));
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert_eq!(add(&mut c, "Doc two").code(), Some("paused"));
}

#[test]
fn no_resume_tool() {
    let w = World::new();
    pause(&w, true);
    let mut c = client(&w);
    for n in c.tool_names() {
        for bad in ["pause", "resume", "limit"] {
            assert!(!n.contains(bad), "{n}");
        }
    }
}

#[test]
fn pause_and_resume_live() {
    let w = World::new();
    let mut c = client(&w);
    assert!(!add(&mut c, "Doc one").is_error);
    pause(&w, true);
    assert_eq!(add(&mut c, "Doc two").code(), Some("paused"));
    pause(&w, false);
    let r = add(&mut c, "Doc three");
    assert!(!r.is_error, "{}", r.json);
}

#[test]
fn read_only_wins_over_paused() {
    let w = World::new();
    set(
        &personal_path(&w),
        "[limits]\naccess = \"read-only\"\npaused = true\n",
    );
    let mut c = client(&w);
    assert_eq!(add(&mut c, "Doc one").code(), Some("access_denied"));
}

#[test]
fn paused_hides_write_tools_and_sends_list_changed() {
    let w = World::new();
    let mut c = client(&w);
    assert!(c.tool_names().contains(&"add".to_string()));
    pause(&w, true);
    // A call makes the server see the change.
    c.call("status", json!({}));
    assert_eq!(c.notifications("notifications/tools/list_changed"), 1);
    let names = c.tool_names();
    assert!(!names.contains(&"add".to_string()));
    assert!(names.contains(&"status".to_string()));
    // A hidden write tool is paused, not unknown.
    assert_eq!(add(&mut c, "Doc one").code(), Some("paused"));
    pause(&w, false);
    c.call("status", json!({}));
    assert_eq!(c.notifications("notifications/tools/list_changed"), 2);
    assert!(c.tool_names().contains(&"add".to_string()));
}

#[test]
fn broken_file_is_setup_problem() {
    let w = World::new();
    set(&personal_path(&w), "[limits\npaused = ");
    let mut c = client(&w);
    let r = c.call("list", json!({}));
    assert_eq!(r.code(), Some("setup_problem"));
    assert!(r.message().contains("config.toml") || r.json.to_string().contains("config.toml"));
    let r = add(&mut c, "Doc one");
    assert_eq!(r.code(), Some("setup_problem"));
}

#[test]
fn paused_flag_pauses_from_the_start() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--paused"]));
    let s: Value = c.call("status", json!({})).json;
    assert_eq!(s["settings"]["paused"], true);
    assert_eq!(add(&mut c, "Doc one").code(), Some("paused"));
}

#[test]
fn a_pull_works_while_paused() {
    let w = World::new();
    pause(&w, true);
    let mut c = McpClient::for_person(
        &w,
        Person::A,
        Spawn {
            env: vec![("WAYMARK_FETCH_MAX_AGE".into(), "0".into())],
            ..Spawn::default()
        },
    );
    commit_doc(&w.b, "ops/from-ben", "From Ben", "x");
    git(&w.b, &["push", "-q"]);
    let l = c.call("list", json!({}));
    assert!(!l.is_error, "{}", l.json);
    assert!(l.json.to_string().contains("ops/from-ben"), "{}", l.json);
}
