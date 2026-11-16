//! Git sync with two clones and a bare remote.

mod common;

use common::git::{git, no_operation_in_progress, run, subjects};
use common::{Person, World};
use std::time::{Duration, Instant};
use waymark::error::Code;
use waymark::gitsync::{commit_message, Change, Verb};

const DOC: &str = "---\ntitle: Deploy process\ndescription: How we deploy\nstatus: active\ncreated: 2026-11-01\nupdated: 2026-11-02\ncreated_by: \"A B <a@example.org>\"\nupdated_by: \"A B <a@example.org>\"\n---\nLine one\n";

fn write_doc(
    repo: &waymark::gitsync::Repo,
    rel: &str,
    text: &str,
    verb: Verb,
) -> waymark::error::Result<waymark::gitsync::WriteResult> {
    repo.write(false, |tx| {
        tx.write_file(rel, text)?;
        Ok((
            Change {
                paths: vec![rel.to_string()],
                message: commit_message(verb, "Deploy process"),
            },
            (),
        ))
    })
    .map(|(r, _)| r)
}

#[test]
fn two_writers_different_files() {
    let w = World::new();
    let (a, b) = (w.repo(Person::A), w.repo(Person::B));
    assert!(
        write_doc(&a, "agent-memory/ops/one.md", DOC, Verb::Add)
            .unwrap()
            .pushed
    );
    assert!(
        write_doc(&b, "agent-memory/ops/two.md", DOC, Verb::Add)
            .unwrap()
            .pushed
    );
    let remote_files = git(&w.remote, &["ls-tree", "-r", "--name-only", "main"]);
    assert!(
        remote_files.contains("agent-memory/ops/one.md")
            && remote_files.contains("agent-memory/ops/two.md")
    );
    // A sees both after its next update.
    a.refresh_if_stale();
    a.sync(|_| Ok(Vec::new())).unwrap();
    assert!(w.a.join("agent-memory/ops/two.md").exists());
}

#[test]
fn commits_use_the_author_of_waymark_and_the_fixed_message() {
    let w = World::new();
    // The repository config names another person. waymark must not use it.
    git(&w.a, &["config", "user.name", "Someone Else"]);
    git(&w.a, &["config", "user.email", "else@example.org"]);
    let a = w.repo(Person::A);
    write_doc(&a, "agent-memory/ops/one.md", DOC, Verb::Add).unwrap();
    let line = git(&w.a, &["log", "-1", "--format=%an|%ae|%cn|%ce|%s"]);
    assert_eq!(line.trim(), "Anna Example|anna@example.org|Anna Example|anna@example.org|waymark: add \"Deploy process\"");
}

#[test]
fn a_stale_base_is_rejected() {
    let w = World::new();
    let (a, b) = (w.repo(Person::A), w.repo(Person::B));
    write_doc(&a, "agent-memory/ops/d.md", DOC, Verb::Add).unwrap();
    let v1 = a.last_commit("agent-memory/ops/d.md").unwrap().unwrap();
    // B changes the document and pushes.
    b.sync(|_| Ok(Vec::new())).unwrap();
    write_doc(
        &b,
        "agent-memory/ops/d.md",
        &DOC.replace("Line one", "B text"),
        Verb::Update,
    )
    .unwrap();
    // A writes with the old base.
    let err = a
        .write(false, |tx| {
            tx.check_base("agent-memory/ops/d.md", Some(&v1))?;
            tx.write_file("agent-memory/ops/d.md", "x")?;
            Ok((
                Change {
                    paths: vec!["agent-memory/ops/d.md".into()],
                    message: "x".into(),
                },
                (),
            ))
        })
        .expect_err("must fail");
    assert_eq!(err.code, Code::StaleBase);
    assert!(err.next.is_some());
    // The failed write changed nothing.
    assert!(w.a.join("agent-memory/ops/d.md").exists());
    assert!(std::fs::read_to_string(w.a.join("agent-memory/ops/d.md"))
        .unwrap()
        .contains("B text"));
    assert!(git(&w.a, &["status", "--porcelain"]).trim().is_empty());
}

#[test]
fn a_real_conflict_keeps_both_versions_and_leaves_no_rebase() {
    let w = World::new();
    let (a, b) = (w.repo(Person::A), w.repo(Person::B));
    write_doc(&a, "agent-memory/ops/d.md", DOC, Verb::Add).unwrap();
    b.sync(|_| Ok(Vec::new())).unwrap();
    // B changes the line and pushes.
    write_doc(
        &b,
        "agent-memory/ops/d.md",
        &DOC.replace("Line one", "Line from B"),
        Verb::Update,
    )
    .unwrap();
    // A changes the same line, but only commits locally (no push).
    std::fs::write(
        w.a.join("agent-memory/ops/d.md"),
        DOC.replace("Line one", "Line from A"),
    )
    .unwrap();
    git(&w.a, &["add", "-A"]);
    git(
        &w.a,
        &["commit", "-q", "-m", "waymark: update \"Deploy process\""],
    );
    let result = a.sync(|_| Ok(Vec::new())).unwrap();
    assert!(
        no_operation_in_progress(&w.a),
        "no rebase or merge may stay"
    );
    assert_eq!(result.conflicts.len(), 1);
    assert!(result
        .notices
        .iter()
        .any(|n| n.contains("Two people changed")));
    // The remote version is at the original path.
    let orig = std::fs::read_to_string(w.a.join("agent-memory/ops/d.md")).unwrap();
    assert!(orig.contains("Line from B") && !orig.contains("Line from A"));
    // The local version is in a disputed copy.
    let copy_path = &result.conflicts[0].1;
    let copy = std::fs::read_to_string(w.a.join(copy_path)).unwrap();
    assert!(copy.contains("Line from A"));
    assert!(copy.contains("disputed: true"));
    assert!(copy.contains("[conflict "));
    // Both are on the remote.
    assert!(result.pushed);
    let remote_files = git(&w.remote, &["ls-tree", "-r", "--name-only", "main"]);
    assert!(remote_files.contains("-conflict-"));
    // The copy passes the document checks.
    let rel = copy_path.strip_prefix("agent-memory/").unwrap();
    let findings = waymark::format::check(rel, &copy, &waymark::format::Limits::default(), None);
    assert!(!waymark::format::has_errors(&findings), "{findings:?}");
}

#[test]
fn a_rejected_push_is_not_retried_and_the_commit_stays_local() {
    let w = World::new();
    let hook = w.remote.join("hooks/pre-receive");
    std::fs::write(&hook, "#!/bin/sh\necho no >&2\nexit 1\n").unwrap();
    run(&w.root, &["config", "--global", "x.y", "z"]); // no effect: global config is /dev/null
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let a = w.repo(Person::A);
    let err = write_doc(&a, "agent-memory/ops/one.md", DOC, Verb::Add).expect_err("push must fail");
    assert_eq!(err.code, Code::SyncFailed);
    assert!(err.message.contains("rejected"));
    assert!(err.next.is_some());
    // The commit is saved on this computer.
    assert_eq!(subjects(&w.a)[0], "waymark: add \"Deploy process\"");
    assert!(no_operation_in_progress(&w.a));
}

#[test]
fn preview_writes_nothing() {
    let w = World::new();
    let a = w.repo(Person::A);
    let before = git(&w.a, &["rev-parse", "HEAD"]);
    let (r, _) = a
        .write(true, |tx| {
            tx.write_file("agent-memory/ops/new.md", DOC)?;
            Ok((
                Change {
                    paths: vec!["agent-memory/ops/new.md".into()],
                    message: "x".into(),
                },
                (),
            ))
        })
        .unwrap();
    assert!(r.preview && r.commit.is_none());
    assert_eq!(git(&w.a, &["rev-parse", "HEAD"]), before);
    assert!(!w.a.join("agent-memory/ops/new.md").exists());
    assert!(git(&w.a, &["status", "--porcelain"]).trim().is_empty());
}

#[test]
fn a_failed_closure_restores_the_files() {
    let w = World::new();
    let a = w.repo(Person::A);
    write_doc(&a, "agent-memory/ops/d.md", DOC, Verb::Add).unwrap();
    let err = a
        .write(false, |tx| {
            tx.write_file("agent-memory/ops/d.md", "changed")?;
            tx.write_file("agent-memory/ops/other.md", "new")?;
            Err::<(Change, ()), _>(waymark::error::Error::validation("no"))
        })
        .err()
        .unwrap();
    assert_eq!(err.code, Code::ValidationFailed);
    assert_eq!(
        std::fs::read_to_string(w.a.join("agent-memory/ops/d.md")).unwrap(),
        DOC
    );
    assert!(!w.a.join("agent-memory/ops/other.md").exists());
}

#[test]
fn the_same_write_twice_changes_nothing_the_second_time() {
    let w = World::new();
    let a = w.repo(Person::A);
    write_doc(&a, "agent-memory/ops/d.md", DOC, Verb::Add).unwrap();
    let count = subjects(&w.a).len();
    let r = write_doc(&a, "agent-memory/ops/d.md", DOC, Verb::Add).unwrap();
    assert!(r.unchanged && r.commit.is_none());
    assert_eq!(subjects(&w.a).len(), count);
}

#[test]
fn only_the_paths_of_the_write_are_committed() {
    let w = World::new();
    let a = w.repo(Person::A);
    // A manual edit that is not part of the write.
    std::fs::write(w.a.join("agent-memory/manual.md"), "x").unwrap();
    write_doc(&a, "agent-memory/ops/d.md", DOC, Verb::Add).unwrap();
    let files = git(&w.a, &["show", "--name-only", "--format=", "HEAD"]);
    assert_eq!(files.trim(), "agent-memory/ops/d.md");
    assert!(w.a.join("agent-memory/manual.md").exists());
}

#[test]
fn sync_waits_for_the_lock() {
    let w = World::new();
    let a = std::sync::Arc::new(w.repo(Person::A));
    let a2 = a.clone();
    let start = Instant::now();
    let holder = std::thread::spawn(move || {
        a2.write(false, |tx| {
            std::thread::sleep(Duration::from_millis(500));
            tx.write_file("agent-memory/ops/d.md", DOC)?;
            Ok((
                Change {
                    paths: vec!["agent-memory/ops/d.md".into()],
                    message: commit_message(Verb::Add, "x"),
                },
                (),
            ))
        })
        .unwrap();
    });
    std::thread::sleep(Duration::from_millis(100));
    a.sync(|_| Ok(Vec::new())).unwrap();
    assert!(
        start.elapsed() >= Duration::from_millis(450),
        "sync must wait for the lock"
    );
    holder.join().unwrap();
}

#[test]
fn a_repository_without_a_remote_works_with_a_notice() {
    let w = World::new();
    git(&w.a, &["remote", "remove", "origin"]);
    let a = w.repo(Person::A);
    let r = write_doc(&a, "agent-memory/ops/d.md", DOC, Verb::Add).unwrap();
    assert!(r.commit.is_some() && !r.pushed);
    assert!(r.notices.iter().any(|n| n.contains("no remote")));
}

#[test]
fn a_read_without_network_gives_a_notice_and_does_not_fail() {
    let w = World::new();
    let a = w.repo(Person::A); // opened fresh: the last fetch is unknown
    std::fs::rename(&w.remote, w.root.join("gone.git")).unwrap();
    let notices = a.refresh_if_stale();
    assert!(
        notices.iter().any(|n| n.contains("may be behind")),
        "{notices:?}"
    );
    // A write fails clearly instead.
    let err = write_doc(&a, "agent-memory/ops/d.md", DOC, Verb::Add)
        .err()
        .unwrap();
    assert_eq!(err.code, Code::SyncFailed);
}

#[test]
fn a_clone_from_a_url_happens_once() {
    let w = World::new();
    let dir = w.root.join("c");
    let author = World::author(Person::A);
    let repo =
        waymark::gitsync::Repo::clone_from(w.remote.to_str().unwrap(), &dir, author).unwrap();
    assert!(dir.join("agent-memory/.waymark/settings.toml").exists());
    assert!(repo.has_remote());
}

#[test]
fn a_missing_remote_fails_fast_without_a_prompt() {
    let w = World::new();
    let start = Instant::now();
    let err = waymark::gitsync::Repo::clone_from(
        w.root.join("nope.git").to_str().unwrap(),
        &w.root.join("d"),
        World::author(Person::A),
    )
    .err()
    .unwrap();
    assert_eq!(err.code, Code::SyncFailed);
    assert!(start.elapsed() < Duration::from_secs(10));
}

#[test]
fn history_helpers() {
    let w = World::new();
    let a = w.repo(Person::A);
    write_doc(&a, "agent-memory/ops/d.md", DOC, Verb::Add).unwrap();
    let log = a.log(5, Some("anna@example.org"), &[]).unwrap();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].subject, "waymark: add \"Deploy process\"");
    assert_eq!(
        a.files_of(&log[0].hash).unwrap(),
        vec![('A', "agent-memory/ops/d.md".to_string())]
    );
    assert_eq!(
        a.show_at(&log[0].hash, "agent-memory/ops/d.md")
            .unwrap()
            .unwrap(),
        DOC
    );
    assert!(a.last_commit("agent-memory/ops/none.md").unwrap().is_none());
}
