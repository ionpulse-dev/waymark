//! End-to-end tests of project binding (see `specs/features/project-binding/test.md`).

mod common;
use common::git::{git, subjects};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const AUTHOR: [&str; 4] = [
    "--author-name",
    "Anna Example",
    "--author-email",
    "anna@example.org",
];

fn home(w: &World, name: &str) -> PathBuf {
    let h = w.root.join(name);
    std::fs::create_dir_all(&h).unwrap();
    h
}

fn start(home: &Path, args: &[&str], cwd: Option<PathBuf>) -> McpClient {
    let mut a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    a.extend(AUTHOR.iter().map(|s| s.to_string()));
    McpClient::start(
        home,
        Spawn {
            args: a,
            cwd,
            ..Spawn::default()
        },
    )
}

fn url(w: &World) -> String {
    format!("file://{}", w.remote.display())
}

/// `status`, after the first clone (it runs in the background, see `join-memory-repository`).
fn status(c: &mut McpClient) -> Value {
    for _ in 0..100 {
        let s = c.call("status", json!({})).json;
        if s["memory_repository"]["state"] != "cloning" {
            return s;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("the clone did not end");
}

fn memory_repository_id(w: &World) -> String {
    let text = std::fs::read_to_string(w.a.join("agent-memory/.waymark/settings.toml")).unwrap();
    text.lines()
        .find_map(|l| l.strip_prefix("id = "))
        .unwrap()
        .trim_matches('"')
        .to_string()
}

fn add(c: &mut McpClient) -> common::Reply {
    c.call(
        "add",
        json!({"folder": "ops", "title": "Doc one", "description": "D.", "body": "B."}),
    )
}

fn personal(h: &Path, text: &str) {
    let p = h.join(".config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

#[test]
fn repo_argument_clones_to_the_default_folder_once() {
    let w = World::new();
    let h = home(&w, "h1");
    let u = url(&w);
    let mut c = start(&h, &["--repo", &u], None);
    let s = status(&mut c);
    assert_eq!(s["memory_repository"]["found_by"], "repo_argument");
    assert_eq!(s["memory_repository"]["id"], memory_repository_id(&w));
    let dir = h.join("Projects/remote-agent-memory");
    assert!(dir.join(".git").is_dir());
    drop(c);
    // A second start reuses the clone: a marker file in it survives.
    std::fs::write(dir.join("marker.txt"), "x").unwrap();
    let mut c = start(&h, &["--repo", &u], None);
    assert_eq!(status(&mut c)["memory_repository"]["state"], "ready");
    assert!(dir.join("marker.txt").exists());
}

#[test]
fn repo_and_path_clone_into_the_path() {
    let w = World::new();
    let h = home(&w, "h2");
    let target = h.join("elsewhere");
    let u = url(&w);
    let mut c = start(
        &h,
        &["--repo", &u, "--path", target.to_str().unwrap()],
        None,
    );
    let s = status(&mut c);
    assert_eq!(s["memory_repository"]["found_by"], "path_argument");
    assert!(target.join(".git").is_dir());
}

#[test]
fn memrepo_id_equal_works_and_different_serves_nothing() {
    let w = World::new();
    let h = home(&w, "h3");
    let p = w.a.to_str().unwrap();
    let id = memory_repository_id(&w);
    let mut c = start(&h, &["--path", p, "--memory-repository-id", &id], None);
    assert!(!add(&mut c).is_error);
    drop(c);
    let before = subjects(&w.a);
    let mut c = start(
        &h,
        &[
            "--path",
            p,
            "--memory-repository-id",
            "00000000-0000-4000-8000-000000000000",
        ],
        None,
    );
    let s = status(&mut c);
    assert_eq!(s["healthy"], false);
    assert_eq!(c.call("list", json!({})).code(), Some("setup_problem"));
    assert_eq!(
        c.call(
            "add",
            json!({"folder": "ops", "title": "T", "description": "D.", "body": "B."})
        )
        .code(),
        Some("setup_problem")
    );
    assert_eq!(subjects(&w.a), before);
}

#[test]
fn a_missing_id_is_a_setup_problem() {
    let w = World::new();
    let h = home(&w, "h4");
    let f = w.a.join("agent-memory/.waymark/settings.toml");
    let t = std::fs::read_to_string(&f).unwrap();
    let t: String = t
        .lines()
        .filter(|l| !l.starts_with("id = "))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&f, t).unwrap();
    let mut c = start(&h, &["--path", w.a.to_str().unwrap()], None);
    assert_eq!(c.call("list", json!({})).code(), Some("setup_problem"));
}

#[test]
fn an_id_that_changes_after_a_pull_stops_the_tools() {
    let w = World::new();
    let h = home(&w, "h5");
    let id = memory_repository_id(&w);
    let mut c = start(
        &h,
        &[
            "--path",
            w.a.to_str().unwrap(),
            "--memory-repository-id",
            &id,
        ],
        None,
    );
    assert!(!c.call("list", json!({})).is_error);
    let f = w.a.join("agent-memory/.waymark/settings.toml");
    let t = std::fs::read_to_string(&f)
        .unwrap()
        .replace(&id, "00000000-0000-4000-8000-000000000001");
    std::fs::write(&f, t).unwrap();
    assert_eq!(c.call("list", json!({})).code(), Some("setup_problem"));
}

#[test]
fn allowed_hosts_refuse_a_clone_and_allow_localhost() {
    let w = World::new();
    let h = home(&w, "h6");
    personal(&h, "[limits]\nallowed_repo_hosts = [\"git.example.org\"]\n");
    let u = url(&w);
    let mut c = start(&h, &["--repo", &u], None);
    assert_eq!(c.call("list", json!({})).code(), Some("access_denied"));
    let dir = h.join("Projects/remote-agent-memory");
    assert!(!dir.exists());
    assert!(!h
        .join("Projects/remote-agent-memory.waymark-clone")
        .exists());
    drop(c);
    personal(&h, "[limits]\nallowed_repo_hosts = [\"localhost\"]\n");
    let mut c = start(&h, &["--repo", &u], None);
    assert!(!c.call("list", json!({})).is_error);
}

#[test]
fn an_existing_folder_with_a_refused_origin_is_denied_and_no_origin_passes() {
    let w = World::new();
    let h = home(&w, "h7");
    personal(&h, "[limits]\nallowed_repo_hosts = [\"git.example.org\"]\n");
    let mut c = start(&h, &["--path", w.a.to_str().unwrap()], None);
    assert_eq!(c.call("list", json!({})).code(), Some("access_denied"));
    drop(c);
    git(&w.a, &["remote", "remove", "origin"]);
    let mut c = start(&h, &["--path", w.a.to_str().unwrap()], None);
    assert!(!c.call("list", json!({})).is_error);
}

#[test]
fn tools_take_no_repository_argument() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    for t in c.list_tools() {
        let schema = &t["inputSchema"];
        assert_eq!(schema["additionalProperties"], false, "{}", t["name"]);
        for bad in ["repo", "path", "memrepo", "memory_repository_id"] {
            assert!(
                schema["properties"].get(bad).is_none(),
                "{} {bad}",
                t["name"]
            );
        }
    }
    let r = c.call("list", json!({"repo": "file:///elsewhere"}));
    assert_eq!(r.code(), Some("invalid_arguments"));
}

#[test]
fn working_folder_is_found_from_the_root_a_subfolder_and_the_parent() {
    let w = World::new();
    let h = home(&w, "h8");
    for cwd in [
        w.a.join("agent-memory"),
        w.a.join("agent-memory/.waymark"),
        w.a.clone(),
    ] {
        let mut c = start(&h, &[], Some(cwd));
        let s = status(&mut c);
        assert_eq!(s["memory_repository"]["found_by"], "working_folder");
    }
}

#[test]
fn single_default_folder_is_used() {
    let w = World::new();
    let h = home(&w, "h9");
    git(
        &w.root,
        &[
            "clone",
            "-q",
            w.remote.to_str().unwrap(),
            h.join("Projects/a-agent-memory").to_str().unwrap(),
        ],
    );
    let mut c = start(&h, &[], None);
    let s = status(&mut c);
    assert_eq!(s["memory_repository"]["found_by"], "default_folder");
}

#[test]
fn two_default_folders_and_nothing_found() {
    let w = World::new();
    let h = home(&w, "h10");
    let mut c = start(&h, &[], None);
    assert!(status(&mut c).is_object());
    let r = c.call("list", json!({}));
    assert_eq!(r.code(), Some("setup_problem"));
    assert!(r.json["next"].as_str().unwrap().contains("--repo"));
    drop(c);
    for n in ["a", "b"] {
        git(
            &w.root,
            &[
                "clone",
                "-q",
                w.remote.to_str().unwrap(),
                h.join(format!("Projects/{n}-agent-memory"))
                    .to_str()
                    .unwrap(),
            ],
        );
    }
    let mut c = start(&h, &[], None);
    assert!(status(&mut c).is_object());
    let r = c.call("list", json!({}));
    assert_eq!(r.code(), Some("setup_problem"));
    assert!(r.message().contains("a-agent-memory") && r.message().contains("b-agent-memory"));
    assert!(r.json["next"].as_str().unwrap().contains("--path"));
}

#[test]
fn a_git_repository_without_a_marker() {
    let w = World::new();
    let h = home(&w, "h11");
    let d = h.join("plain");
    std::fs::create_dir_all(&d).unwrap();
    git(&d, &["init", "-q"]);
    let mut c = start(&h, &["--path", d.to_str().unwrap()], None);
    assert!(status(&mut c).is_object());
    assert_eq!(c.call("list", json!({})).code(), Some("setup_problem"));
}

#[test]
fn origin_mismatch_does_not_touch_the_folder() {
    let w = World::new();
    let h = home(&w, "h12");
    let before = git(&w.a, &["remote", "-v"]);
    let mut c = start(
        &h,
        &[
            "--repo",
            "file:///somewhere/else.git",
            "--path",
            w.a.to_str().unwrap(),
        ],
        None,
    );
    let r = c.call("list", json!({}));
    assert_eq!(r.code(), Some("setup_problem"));
    assert_eq!(git(&w.a, &["remote", "-v"]), before);
}

#[test]
fn a_linked_worktree_and_a_non_repository() {
    let w = World::new();
    let h = home(&w, "h13");
    let wt = w.root.join("wt");
    git(
        &w.a,
        &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "other"],
    );
    let mut c = start(&h, &["--path", wt.to_str().unwrap()], None);
    assert_eq!(c.call("list", json!({})).code(), Some("setup_problem"));
    drop(c);
    let plain = h.join("not-git");
    std::fs::create_dir_all(&plain).unwrap();
    let mut c = start(&h, &["--path", plain.to_str().unwrap()], None);
    assert_eq!(c.call("list", json!({})).code(), Some("setup_problem"));
    drop(c);
    let mut c = start(&h, &["--path", h.join("missing").to_str().unwrap()], None);
    assert_eq!(c.call("list", json!({})).code(), Some("setup_problem"));
}

#[test]
fn a_failed_clone_leaves_no_temporary_folder() {
    let w = World::new();
    let h = home(&w, "h14");
    // A leftover from an earlier start.
    let tmp = h.join("Projects/nothing-agent-memory.waymark-clone");
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("junk"), "x").unwrap();
    let mut c = start(&h, &["--repo", "file:///does/not/exist/nothing.git"], None);
    let r = c.call("list", json!({}));
    assert_eq!(r.code(), Some("sync_failed"), "{}", r.json);
    assert!(!tmp.exists());
    assert!(!h.join("Projects/nothing-agent-memory").exists());
}

#[test]
fn two_servers_do_not_mix() {
    let w = World::new();
    let h = home(&w, "h15");
    let mut a = McpClient::for_person(&w, Person::A, Spawn::default());
    let mut b = McpClient::for_person(&w, Person::B, Spawn::default());
    let (ra, rb) = (subjects(&w.b), subjects(&w.a));
    assert!(!add(&mut a).is_error);
    assert_eq!(subjects(&w.b), ra);
    assert!(subjects(&w.a).len() > rb.len());
    let _ = (&mut b, h);
}

#[test]
fn a_url_with_a_password_is_never_shown() {
    let w = World::new();
    let h = home(&w, "h16");
    let bad = "https://anna:secret123@127.0.0.1:9/none/x.git";
    let mut c = start(&h, &["--repo", bad], None);
    let s = status(&mut c);
    let r = c.call("list", json!({}));
    for text in [s.to_string(), r.json.to_string()] {
        assert!(!text.contains("secret123"), "{text}");
    }
}
