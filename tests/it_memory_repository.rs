//! End-to-end tests of the memory repository layout (see `specs/foundation/memory-repository/test.md`).

mod common;
use common::git::git;
use common::{McpClient, Person, Spawn, World};
use serde_json::json;

fn check<'a>(r: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap()
}

#[test]
fn gitignore_hides_personal_obsidian_files() {
    let w = World::new();
    let obsidian = w.a.join(".obsidian");
    std::fs::write(obsidian.join("workspace.json"), "{}").unwrap();
    std::fs::write(obsidian.join("graph.json"), "{}").unwrap();
    assert_eq!(git(&w.a, &["status", "--porcelain"]), "");
    // The shared file stays in Git.
    assert!(git(&w.a, &["ls-files"]).contains(".obsidian/app.json"));
}

#[test]
fn init_writes_obsidian_defaults() {
    let w = World::new();
    let app: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(w.a.join(".obsidian/app.json")).unwrap())
            .unwrap();
    assert_eq!(app["useMarkdownLinks"], true);
    assert_eq!(app["newLinkFormat"], "relative");
}

#[test]
fn missing_id_is_setup_problem() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let without: String = text
        .lines()
        .filter(|l| !l.starts_with("id ="))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, without).unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = c.call("status", json!({}));
    assert_eq!(r.json["healthy"], false);
    let memrepo = check(&r.json, "clone");
    assert_eq!(memrepo["ok"], false);
    assert!(memrepo["detail"].as_str().unwrap().contains("id"));
}

#[test]
fn stricter_value_applies() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        text.replace("[limits]", "[limits]\nmax_tokens = 400"),
    )
    .unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--max-tokens", "900"]));
    assert_eq!(
        c.call("status", json!({})).json["settings"]["max_tokens"],
        400
    );
}

#[test]
fn unknown_keys_give_a_notice_and_no_error() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, format!("{text}\n[future]\nthing = 1\n")).unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = c.call("status", json!({}));
    assert!(!r.is_error);
    assert_eq!(r.json["healthy"], true);
    assert!(r.json["notices"]
        .as_array()
        .map(|n| !n.is_empty())
        .unwrap_or(false));
}

#[test]
fn the_layout_has_the_root_in_agent_memory_and_no_owners_field() {
    let w = World::new();
    let memrepo = waymark::memrepo::MemRepo::open(&w.a).unwrap();
    assert!(memrepo.root.ends_with("agent-memory"));
    assert!(memrepo.documents().is_empty());
    let toml = std::fs::read_to_string(w.a.join("agent-memory/.waymark/settings.toml")).unwrap();
    assert!(!toml.contains("owners"));
}

#[test]
fn a_second_person_gets_the_same_memory_repository_id() {
    let w = World::new();
    let mut a = McpClient::for_person(&w, Person::A, Spawn::default());
    let mut b = McpClient::for_person(&w, Person::B, Spawn::default());
    let ra = a.call("status", json!({}));
    let rb = b.call("status", json!({}));
    assert_eq!(
        ra.json["memory_repository"]["id"],
        rb.json["memory_repository"]["id"]
    );
}

#[test]
fn init_writes_the_schema() {
    let w = World::new();
    let text = std::fs::read_to_string(w.a.join("agent-memory/.waymark/settings.toml")).unwrap();
    assert!(text.lines().any(|l| l.trim() == "schema = 1"), "{text}");
}

#[test]
fn a_higher_schema_blocks_every_tool_but_status() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replace("schema = 1", "schema = 2")).unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    for (tool, args) in [
        ("search", json!({})),
        ("search", json!({"query": "x"})),
        (
            "add_document",
            json!({"folder": "ops", "title": "T", "description": "D.", "body": "B."}),
        ),
    ] {
        let r = c.call(tool, args);
        assert_eq!(r.code(), Some("setup_problem"), "{tool}: {}", r.json);
        assert!(
            r.message().contains("needs a newer waymark"),
            "{tool}: {}",
            r.json
        );
        assert!(r.json["next"]
            .as_str()
            .unwrap()
            .contains(waymark::platform::update_sentence()));
    }
    let s = c.call("status", json!({}));
    assert!(!s.is_error, "{}", s.json);
    assert_eq!(s.json["healthy"], false);
    assert!(
        s.json.to_string().contains("needs a newer waymark"),
        "{}",
        s.json
    );
    assert!(s
        .json
        .to_string()
        .contains(waymark::platform::update_sentence()));
}

#[test]
fn a_file_without_schema_works_as_schema_1() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let without: String = text
        .lines()
        .filter(|l| !l.starts_with("schema"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, without).unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = c.call("search", json!({}));
    assert!(!r.is_error, "{}", r.json);
}

fn remove_schema_and_commit(w: &World) {
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let without: String = text
        .lines()
        .filter(|l| !l.starts_with("schema"))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&path, without).unwrap();
    git(&w.a, &["add", "-A"]);
    git(
        &w.a,
        &[
            "-c",
            "user.name=Host",
            "-c",
            "user.email=host@corp.org",
            "commit",
            "-q",
            "-m",
            "old settings",
        ],
    );
    git(&w.a, &["push", "-q", "origin", "HEAD"]);
}

#[test]
fn a_write_repairs_a_missing_schema_in_its_own_commit() {
    let w = World::new();
    remove_schema_and_commit(&w);
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    // Reads and previews never write.
    let head = git(&w.remote, &["rev-parse", "main"]);
    assert!(!c.call("search", json!({})).is_error);
    assert!(!c
        .call(
            "add_document",
            json!({"folder": "ops", "title": "T", "description": "D.", "body": "B.", "preview": true})
        )
        .is_error);
    assert_eq!(git(&w.remote, &["rev-parse", "main"]), head);
    // A write repairs first, then makes its own commit.
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "After repair", "description": "D.", "body": "B."}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert!(
        r.json.to_string().contains("added the missing schema key"),
        "{}",
        r.json
    );
    let log = git(&w.remote, &["log", "--format=%s", "-3", "main"]);
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(lines[0], "waymark: add \"After repair\"", "{log}");
    assert_eq!(lines[1], "waymark: repair \"settings\"", "{log}");
    let settings = git(
        &w.remote,
        &["show", "main:agent-memory/.waymark/settings.toml"],
    );
    // A comment on top stays on top, and the key comes before the first setting.
    assert!(
        settings.lines().next().unwrap().starts_with('#'),
        "{settings}"
    );
    let schema_at = settings.find("\nschema = 1\n").expect("the key");
    let name_at = settings.find("\nname = ").expect("a name");
    assert!(schema_at < name_at, "{settings}");
    // A second write has nothing to repair.
    let before = git(&w.remote, &["rev-list", "--count", "main"]);
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Again", "description": "D.", "body": "B."}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert!(!r.json.to_string().contains("Repaired"));
    let after = git(&w.remote, &["rev-list", "--count", "main"]);
    assert_eq!(
        after.trim().parse::<i32>().unwrap(),
        before.trim().parse::<i32>().unwrap() + 1
    );
}

#[test]
fn a_repair_keeps_the_values_that_the_owner_set() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let edited = text.replace("# max_results = 5", "max_results = 2 # tight");
    assert_ne!(edited, text);
    std::fs::write(&path, edited).unwrap();
    remove_schema_and_commit(&w);
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Keep limits", "description": "D.", "body": "B."}),
    );
    assert!(!r.is_error, "{}", r.json);
    let settings = git(
        &w.remote,
        &["show", "main:agent-memory/.waymark/settings.toml"],
    );
    assert!(settings.contains("max_results = 2 # tight"), "{settings}");
    assert!(
        settings.contains("\nschema = 1\n") || settings.starts_with("schema = 1\n"),
        "{settings}"
    );
}

#[test]
fn the_template_explains_every_setting_and_keeps_no_wrong_header() {
    let w = World::new();
    let text = std::fs::read_to_string(w.a.join("agent-memory/.waymark/settings.toml")).unwrap();
    assert!(!text.contains("Every key in the sections below is optional"));
    assert!(text.contains("Identifies this memory repository. Required."));
    // Every line that sets a key has a comment line right above it.
    let lines: Vec<&str> = text.lines().collect();
    for (i, l) in lines.iter().enumerate() {
        let is_key = !l.starts_with('#') && l.contains(" = ") && !l.starts_with('[');
        if is_key {
            assert!(lines[i - 1].starts_with('#'), "no comment above: {l}");
        }
    }
}

// clean-git-status: the repairs of `.gitignore` and `.obsidian/app.json`.

const OLD_GITIGNORE: &str = ".obsidian/*\n!.obsidian/app.json\n";
const OLD_APP_JSON: &str =
    "{\n  \"useMarkdownLinks\": true,\n  \"newLinkFormat\": \"relative\"\n}\n";
const NEW_APP_JSON: &str = "{\n  \"useMarkdownLinks\": true,\n  \"newLinkFormat\": \"relative\"\n}";

/// Write (or remove) files in the clone of person A, commit them as an older tool would, and push.
fn commit_files(w: &World, files: &[(&str, Option<&str>)]) {
    for (path, text) in files {
        let p = w.a.join(path);
        match text {
            Some(t) => std::fs::write(&p, t).unwrap(),
            None => std::fs::remove_file(&p).unwrap(),
        }
    }
    git(&w.a, &["add", "-A"]);
    git(
        &w.a,
        &[
            "-c",
            "user.name=Host",
            "-c",
            "user.email=host@corp.org",
            "commit",
            "-q",
            "-m",
            "old files",
        ],
    );
    git(&w.a, &["push", "-q", "origin", "HEAD"]);
}

fn add(c: &mut McpClient, title: &str) -> common::Reply {
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": title, "description": "D.", "body": "B."}),
    );
    assert!(!r.is_error, "{}", r.json);
    r
}

fn remote_subjects(w: &World, n: usize) -> Vec<String> {
    git(&w.remote, &["log", "--format=%s", &format!("-{n}"), "main"])
        .lines()
        .map(String::from)
        .collect()
}

fn remote_file(w: &World, path: &str) -> String {
    git(&w.remote, &["show", &format!("main:{path}")])
}

// A3
#[test]
fn a_write_adds_ds_store_to_an_old_gitignore() {
    let w = World::new();
    commit_files(&w, &[(".gitignore", Some(OLD_GITIGNORE))]);
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    // Reads and previews never write.
    let head = git(&w.remote, &["rev-parse", "main"]);
    assert!(!c.call("search", json!({})).is_error);
    assert!(!c
        .call(
            "add_document",
            json!({"folder": "ops", "title": "T", "description": "D.", "body": "B.", "preview": true})
        )
        .is_error);
    assert_eq!(git(&w.remote, &["rev-parse", "main"]), head);
    // A write repairs first, then makes its own commit.
    let r = add(&mut c, "After gitignore repair");
    assert!(
        r.json
            .to_string()
            .contains("Git now ignores the .DS_Store files"),
        "{}",
        r.json
    );
    let subjects = remote_subjects(&w, 2);
    assert_eq!(subjects[0], "waymark: add \"After gitignore repair\"");
    assert_eq!(subjects[1], "waymark: repair \".gitignore\"");
    let ignore = remote_file(&w, ".gitignore");
    assert!(ignore.starts_with(OLD_GITIGNORE), "{ignore}");
    assert!(ignore.ends_with("\n.DS_Store\n"), "{ignore}");
    // A second write has nothing to repair.
    let r = add(&mut c, "Again");
    assert!(!r.json.to_string().contains("Repaired"), "{}", r.json);
    assert_eq!(
        remote_subjects(&w, 2)[1],
        "waymark: add \"After gitignore repair\""
    );
}

// A3
#[test]
fn a_gitignore_with_a_glob_for_ds_store_needs_no_repair() {
    let w = World::new();
    commit_files(
        &w,
        &[(
            ".gitignore",
            Some(".obsidian/*\n!.obsidian/app.json\n**/.DS_Store\n"),
        )],
    );
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = add(&mut c, "No repair");
    assert!(!r.json.to_string().contains("Repaired"), "{}", r.json);
    assert_eq!(remote_subjects(&w, 2)[1], "old files");
}

// A4
#[test]
fn a_write_gives_an_old_app_json_the_bytes_of_obsidian() {
    let w = World::new();
    commit_files(&w, &[(".obsidian/app.json", Some(OLD_APP_JSON))]);
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = add(&mut c, "After app repair");
    assert!(
        r.json
            .to_string()
            .contains("the format that Obsidian writes"),
        "{}",
        r.json
    );
    let subjects = remote_subjects(&w, 2);
    assert_eq!(subjects[0], "waymark: add \"After app repair\"");
    assert_eq!(subjects[1], "waymark: repair \".obsidian/app.json\"");
    assert_eq!(remote_file(&w, ".obsidian/app.json"), NEW_APP_JSON);
    let r = add(&mut c, "Again");
    assert!(!r.json.to_string().contains("Repaired"), "{}", r.json);
}

// A5
#[test]
fn a_save_of_obsidian_is_committed_by_the_repair() {
    let w = World::new();
    commit_files(&w, &[(".obsidian/app.json", Some(OLD_APP_JSON))]);
    // Obsidian saves its own form.
    std::fs::write(w.a.join(".obsidian/app.json"), NEW_APP_JSON).unwrap();
    assert!(git(&w.a, &["status", "--porcelain"]).contains("app.json"));
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    add(&mut c, "After Obsidian save");
    assert_eq!(
        remote_subjects(&w, 2)[1],
        "waymark: repair \".obsidian/app.json\""
    );
    assert_eq!(remote_file(&w, ".obsidian/app.json"), NEW_APP_JSON);
    assert_eq!(git(&w.a, &["status", "--porcelain"]), "");
}

// A6
#[test]
fn files_of_the_person_are_left_alone() {
    let w = World::new();
    let own_app =
        "{\n  \"useMarkdownLinks\": true,\n  \"newLinkFormat\": \"relative\",\n  \"vimMode\": true\n}";
    commit_files(
        &w,
        &[
            (".obsidian/app.json", Some(own_app)),
            (".gitignore", Some(OLD_GITIGNORE)),
        ],
    );
    // An uncommitted edit of the person in `.gitignore`.
    let edited = format!("{OLD_GITIGNORE}tmp/\n");
    std::fs::write(w.a.join(".gitignore"), &edited).unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = add(&mut c, "Leave alone");
    assert!(!r.json.to_string().contains("Repaired"), "{}", r.json);
    assert_eq!(remote_file(&w, ".obsidian/app.json"), own_app);
    assert_eq!(remote_file(&w, ".gitignore"), OLD_GITIGNORE);
    assert_eq!(
        std::fs::read_to_string(w.a.join(".gitignore")).unwrap(),
        edited
    );
    assert!(git(&w.a, &["status", "--porcelain"]).contains(".gitignore"));
}

// A7
#[test]
fn a_missing_file_is_not_created() {
    let w = World::new();
    commit_files(&w, &[(".gitignore", None), (".obsidian/app.json", None)]);
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = add(&mut c, "No new files");
    assert!(!r.json.to_string().contains("Repaired"), "{}", r.json);
    let files = git(&w.remote, &["ls-tree", "-r", "--name-only", "main"]);
    assert!(!files.lines().any(|l| l == ".gitignore"), "{files}");
    assert!(!files.lines().any(|l| l == ".obsidian/app.json"), "{files}");
}

// A8
#[test]
fn read_only_access_never_repairs() {
    let w = World::new();
    commit_files(
        &w,
        &[
            (".gitignore", Some(OLD_GITIGNORE)),
            (".obsidian/app.json", Some(OLD_APP_JSON)),
        ],
    );
    let head = git(&w.remote, &["rev-parse", "main"]);
    let mut ro = McpClient::for_person(&w, Person::A, Spawn::args(&["--access", "read-only"]));
    assert!(!ro.call("search", json!({})).is_error);
    assert!(
        ro.call(
            "add_document",
            json!({"folder": "ops", "title": "T", "description": "D.", "body": "B."})
        )
        .is_error
    );
    assert_eq!(git(&w.remote, &["rev-parse", "main"]), head);
    assert_eq!(git(&w.a, &["status", "--porcelain"]), "");
}
