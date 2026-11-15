//! End-to-end tests of `create-memory-repository` (see `specs/features/create-memory-repository/test.md`).

mod common;
use common::git::{git, subjects};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::path::Path;

fn client(w: &World, p: Person) -> McpClient {
    McpClient::for_person(w, p, Spawn::default())
}

fn init(c: &mut McpClient, args: Value) -> common::Reply {
    c.call("init", args)
}

fn remote_files(w: &World) -> Vec<String> {
    git(&w.remote, &["ls-tree", "-r", "--name-only", "main"])
        .lines()
        .map(String::from)
        .collect()
}

fn remote_show(w: &World, path: &str) -> String {
    git(&w.remote, &["show", &format!("main:{path}")])
}

fn remote_head(w: &World) -> String {
    let (ok, out, _) = common::git::run(&w.remote, &["rev-parse", "main"]);
    if ok {
        out.trim().to_string()
    } else {
        String::new()
    }
}

fn commit_all(dir: &Path, msg: &str) {
    git(dir, &["add", "-A"]);
    git(
        dir,
        &[
            "-c",
            "user.name=Host",
            "-c",
            "user.email=host@corp.org",
            "commit",
            "-q",
            "-m",
            msg,
        ],
    );
    git(dir, &["push", "-q", "-u", "origin", "HEAD"]);
}

fn personal(w: &World, text: &str) {
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

#[test]
fn a1_init_creates_the_skeleton_in_one_commit() {
    let w = World::empty();
    let mut c = client(&w, Person::A);
    let r = init(
        &mut c,
        json!({"name": "Example", "slug": "example", "email_domains": ["example.org"],
               "audience": "team", "folders": ["backend", "ops"], "limits": {"max_results": 5}}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["pushed"], true);
    let files = remote_files(&w);
    for f in [
        "AGENTS.md",
        ".gitignore",
        ".obsidian/app.json",
        "agent-memory/.waymark/version",
        "agent-memory/.waymark/settings.toml",
        "agent-memory/.waymark/exclude",
        "agent-memory/backend/.gitkeep",
        "agent-memory/ops/.gitkeep",
    ] {
        assert!(files.contains(&f.to_string()), "{f} missing in {files:?}");
    }
    let listed: Vec<String> = r.json["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(listed.len(), files.len());
    assert_eq!(subjects(&w.remote), vec!["waymark: init \"Example\""]);
    assert_eq!(
        git(&w.remote, &["log", "-1", "--format=%ae", "main"]).trim(),
        "anna@example.org"
    );
    assert_eq!(remote_show(&w, "agent-memory/.waymark/version"), "1.0\n");
    let toml = remote_show(&w, "agent-memory/.waymark/settings.toml");
    let s = waymark::memrepo::settings::parse(&toml, "settings.toml").unwrap();
    assert_eq!(s.name, "Example");
    assert_eq!(s.limits.max_results, Some(5));
    assert!(uuid_ok(&s.id));
    assert_eq!(r.json["id"], s.id);
}

fn uuid_ok(s: &str) -> bool {
    s.len() == 36 && s.chars().filter(|c| *c == '-').count() == 4
}

#[test]
fn a2_a_second_person_needs_only_the_url() {
    let w = World::empty();
    let mut a = client(&w, Person::A);
    assert!(
        !init(
            &mut a,
            json!({"name": "Example", "limits": {"max_results": 2}})
        )
        .is_error
    );
    let mut b = client(&w, Person::B);
    let s = b.call("status", json!({}));
    assert_eq!(s.json["memory_repository"]["state"], "ready", "{}", s.json);
    assert_eq!(s.json["settings"]["max_results"], 2);
}

#[test]
fn a3_init_twice_changes_nothing() {
    let w = World::empty();
    let mut c = client(&w, Person::A);
    assert!(!init(&mut c, json!({"name": "Example"})).is_error);
    let head = remote_head(&w);
    let r = init(&mut c, json!({"name": "Example"}));
    assert_eq!(r.code(), Some("validation_failed"));
    assert!(r.message().contains("already a memory repository"));
    assert_eq!(remote_head(&w), head);
}

#[test]
fn a4_a_repository_with_other_files_is_refused() {
    let w = World::empty();
    std::fs::create_dir_all(w.a.join("src")).unwrap();
    std::fs::write(w.a.join("src/main.rs"), "fn main() {}\n").unwrap();
    commit_all(&w.a, "code");
    let head = remote_head(&w);
    let mut c = client(&w, Person::A);
    let r = init(&mut c, json!({"name": "Example"}));
    assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
    assert_eq!(remote_head(&w), head);
}

#[test]
fn a5_an_existing_agents_file_is_never_overwritten() {
    let w = World::empty();
    std::fs::write(w.a.join("AGENTS.md"), "my own rules\n").unwrap();
    commit_all(&w.a, "agents");
    let mut c = client(&w, Person::A);
    let r = init(&mut c, json!({"name": "Example"}));
    assert_eq!(r.code(), Some("validation_failed"));
    assert_eq!(remote_show(&w, "AGENTS.md"), "my own rules\n");
}

#[test]
fn a6_host_files_stay_as_they_are() {
    let w = World::empty();
    std::fs::write(w.a.join("README.md"), "# Hello\n").unwrap();
    std::fs::write(w.a.join("LICENSE"), "MIT text\n").unwrap();
    commit_all(&w.a, "host files");
    let mut c = client(&w, Person::A);
    let r = init(&mut c, json!({"name": "Example"}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(remote_show(&w, "README.md"), "# Hello\n");
    assert_eq!(remote_show(&w, "LICENSE"), "MIT text\n");
}

#[test]
fn a7_obsidian_files_have_the_exact_content() {
    let w = World::empty();
    let mut c = client(&w, Person::A);
    assert!(!init(&mut c, json!({"name": "Example"})).is_error);
    let app: Value = serde_json::from_str(&remote_show(&w, ".obsidian/app.json")).unwrap();
    assert_eq!(app["useMarkdownLinks"], true);
    assert_eq!(app["newLinkFormat"], "relative");
    let ignore = remote_show(&w, ".gitignore");
    assert!(ignore.lines().any(|l| l == ".obsidian/*"));
    assert!(ignore.lines().any(|l| l == "!.obsidian/app.json"));
    git(&w.a, &["pull", "-q"]);
    std::fs::write(w.a.join(".obsidian/workspace.json"), "{}").unwrap();
    assert!(!git(&w.a, &["status", "--porcelain"]).contains("workspace.json"));
}

#[test]
fn a8_limits_apply_to_init() {
    let w = World::empty();
    personal(&w, "[limits]\naccess = \"read-only\"\n");
    let mut c = client(&w, Person::A);
    assert_eq!(
        init(&mut c, json!({"name": "E"})).code(),
        Some("access_denied")
    );
    drop(c);
    personal(&w, "[limits]\ntools_denied = [\"init\"]\n");
    let mut c = client(&w, Person::A);
    assert!(!c.tool_names().contains(&"init".to_string()));
    drop(c);
    personal(&w, "[limits]\npaused = true\n");
    let mut c = client(&w, Person::A);
    assert_eq!(init(&mut c, json!({"name": "E"})).code(), Some("paused"));
    assert!(remote_head(&w).is_empty());
}

#[test]
fn a9_a_preview_writes_nothing() {
    let w = World::empty();
    let mut c = client(&w, Person::A);
    let r = init(&mut c, json!({"name": "Example", "preview": true}));
    assert!(!r.is_error, "{}", r.json);
    assert!(r.json["commit"].is_null());
    assert!(r.json["id"].is_null());
    assert_eq!(r.json["pushed"], false);
    assert!(!w.a.join("AGENTS.md").exists());
    assert!(!w.a.join("agent-memory").exists());
    assert!(remote_head(&w).is_empty());
}

#[test]
fn a10_the_same_session_can_add_and_search() {
    let w = World::empty();
    let mut c = client(&w, Person::A);
    assert!(!init(&mut c, json!({"name": "Example"})).is_error);
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Deploys use a pipeline", "description": "How deploys run.", "body": "Text."}),
    );
    assert!(!r.is_error, "{}", r.json);
    let s = c.call("search", json!({"query": "deploys"}));
    assert!(!s.is_error, "{}", s.json);
    assert_eq!(
        s.json["results"].as_array().map(|a| a.len()),
        Some(1),
        "{}",
        s.json
    );
}

#[test]
fn a11_bad_input_is_invalid_arguments_and_names_the_field() {
    let w = World::empty();
    let mut c = client(&w, Person::A);
    for (args, field) in [
        (json!({"name": "E", "slug": "Bad Slug"}), "slug"),
        (json!({"name": "E", "folders": ["../x"]}), "folders"),
        (json!({"name": "E", "folders": ["Upper"]}), "folders"),
        (json!({"name": "E", "limits": {"nope": 1}}), "limits"),
        (json!({"name": "   "}), "name"),
        (
            json!({"name": "E", "limits": {"access": "everything"}}),
            "limits",
        ),
    ] {
        let r = init(&mut c, args.clone());
        assert_eq!(r.code(), Some("invalid_arguments"), "{args}: {}", r.json);
        let text = format!("{} {}", r.message(), r.json["next"]);
        assert!(text.contains(field), "{args}: {text}");
    }
    assert!(remote_head(&w).is_empty());
}

#[test]
fn a12_two_calls_at_once_make_one_init_commit() {
    let w = World::empty();
    let mut a = client(&w, Person::A);
    let mut b = client(&w, Person::B);
    let ra = init(&mut a, json!({"name": "Example"}));
    let rb = init(&mut b, json!({"name": "Example"}));
    let ok = [&ra, &rb].iter().filter(|r| !r.is_error).count();
    assert_eq!(ok, 1, "{} / {}", ra.json, rb.json);
    let failed = if ra.is_error { &ra } else { &rb };
    assert_eq!(failed.code(), Some("validation_failed"), "{}", failed.json);
    let inits = subjects(&w.remote)
        .iter()
        .filter(|s| s.starts_with("waymark: init"))
        .count();
    assert_eq!(inits, 1);
}

#[test]
fn a14_agents_md_carries_the_pinned_version() {
    for (aud, name) in [
        ("personal", "personal-agent-memory"),
        ("team", "team-agent-memory"),
    ] {
        let w = World::empty();
        let mut c = client(&w, Person::A);
        assert!(!init(&mut c, json!({"name": "E", "audience": aud})).is_error);
        let text = remote_show(&w, "AGENTS.md");
        let version = waymark::memrepo::agents_file::skill_version(if aud == "team" {
            waymark::memrepo::agents_file::Audience::Team
        } else {
            waymark::memrepo::agents_file::Audience::Personal
        });
        assert!(text.contains(&format!("begin: {name} {version} -->")));
    }
}

#[test]
fn a15_the_written_files_pass_the_checks() {
    let w = World::empty();
    let mut c = client(&w, Person::A);
    assert!(
        !init(
            &mut c,
            json!({"name": "Example", "email_domains": ["example.org"]})
        )
        .is_error
    );
    let r = c.call("lint", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(
        r.json["findings"].as_array().map(|a| a.len()),
        Some(0),
        "{}",
        r.json
    );
    let s = c.call("status", json!({}));
    assert_eq!(s.json["healthy"], true, "{}", s.json);
}

#[test]
fn undo_refuses_an_init_commit() {
    let w = World::empty();
    let mut c = client(&w, Person::A);
    let r = init(&mut c, json!({"name": "Example"}));
    let hash = r.json["commit"].as_str().unwrap().to_string();
    let u = c.call("undo", json!({"commit": hash}));
    assert!(u.is_error, "{}", u.json);
}

/// A folder with the marker and documents but no `settings.toml`.
fn old_marker_repo(w: &World, base: &str) {
    let dir = w.a.join(base);
    std::fs::create_dir_all(dir.join(".waymark")).unwrap();
    std::fs::write(dir.join(".waymark/version"), "1.0\n").unwrap();
    std::fs::write(dir.join(".waymark/exclude"), "README.md\n").unwrap();
    std::fs::create_dir_all(dir.join("work")).unwrap();
    std::fs::write(
        dir.join("work/note.md"),
        "---\ntitle: Note\ndescription: A note.\n---\nText.\n",
    )
    .unwrap();
    std::fs::write(w.a.join("README.md"), "my readme\n").unwrap();
    commit_all(&w.a, "old layout");
}

#[test]
fn a16_a_folder_with_the_marker_and_no_settings_file_gets_only_that_file() {
    let w = World::empty();
    old_marker_repo(&w, "");
    let before = remote_files(&w);
    let mut c = client(&w, Person::A);
    let s = c.call("status", json!({}));
    assert_eq!(s.json["memory_repository"]["state"], "broken", "{}", s.json);
    let r = init(&mut c, json!({"name": "Mine"}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["adopted"], true);
    assert_eq!(r.json["files"], json!([".waymark/settings.toml"]));
    let after = remote_files(&w);
    let added: Vec<_> = after.iter().filter(|f| !before.contains(f)).collect();
    assert_eq!(added, vec![".waymark/settings.toml"]);
    assert_eq!(remote_show(&w, "README.md"), "my readme\n");
    assert_eq!(remote_show(&w, ".waymark/exclude"), "README.md\n");
    assert!(remote_show(&w, ".waymark/settings.toml").contains("name = \"Mine\""));
    // The same session works at once, and a second call is refused.
    let s = c.call("status", json!({}));
    assert_eq!(s.json["memory_repository"]["state"], "ready", "{}", s.json);
    assert!(!c.call("search", json!({})).is_error);
    let again = init(&mut c, json!({"name": "Mine"}));
    assert_eq!(again.code(), Some("validation_failed"));
}

#[test]
fn a17_the_marker_in_agent_memory_gets_the_file_there() {
    let w = World::empty();
    old_marker_repo(&w, "agent-memory");
    let mut c = client(&w, Person::A);
    let r = init(&mut c, json!({"name": "Mine"}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(
        r.json["files"],
        json!(["agent-memory/.waymark/settings.toml"])
    );
}

#[test]
fn a18_folders_and_audience_are_refused_for_a_folder_with_the_marker() {
    let w = World::empty();
    old_marker_repo(&w, "");
    let head = remote_head(&w);
    let mut c = client(&w, Person::A);
    let r = init(&mut c, json!({"name": "Mine", "folders": ["ops"]}));
    assert_eq!(r.code(), Some("invalid_arguments"), "{}", r.json);
    let r = init(&mut c, json!({"name": "Mine", "audience": "personal"}));
    assert_eq!(r.code(), Some("invalid_arguments"), "{}", r.json);
    assert_eq!(remote_head(&w), head);
}
