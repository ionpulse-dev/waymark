//! Tests of `plain-git-access` (see `specs/features/plain-git-access/test.md`).
//!
//! The `init` tool comes with `create-memory-repository`. Until then the tests call `memrepo::init_with`,
//! the function that the tool calls.

use std::path::{Path, PathBuf};
use waymark::memrepo::agents_file::{frontmatter_value, render, skill_version, Audience};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_file(a: Audience) -> String {
    std::fs::read_to_string(
        root()
            .join("plain-git")
            .join(a.skill_name())
            .join("SKILL.md"),
    )
    .unwrap()
}

fn init_in(dir: &Path, a: Audience, domains: &[&str]) -> String {
    let d: Vec<String> = domains.iter().map(|s| s.to_string()).collect();
    waymark::memrepo::init_with(dir, "Test", "test", a, &d).unwrap();
    std::fs::read_to_string(dir.join("AGENTS.md")).unwrap()
}

#[test]
fn a1_init_writes_the_intro_the_rules_and_the_marker() {
    let dir = tempfile::tempdir().unwrap();
    let text = init_in(dir.path(), Audience::Personal, &[]);
    assert_eq!(text, render(Audience::Personal, &[]));
    assert!(text.starts_with("# Rules for agents\n"));
    let version = skill_version(Audience::Personal);
    assert!(text.contains(&format!(
        "<!-- plain-git-rules begin: personal-agent-memory {version} -->"
    )));
    assert!(text.contains("<!-- plain-git-rules end -->"));
    assert!(text.contains("## Rules of this team"));
}

#[test]
fn a2_the_pin_equals_the_version_of_the_file() {
    for a in [Audience::Personal, Audience::Team] {
        let file = skill_file(a);
        let v = frontmatter_value(&file, "version").unwrap();
        assert_eq!(skill_version(a), v);
        assert!(render(a, &[]).contains(&format!("begin: {} {v} -->", a.skill_name())));
    }
}

#[test]
fn a4_personal_has_the_personal_skill_and_no_team_part() {
    let dir = tempfile::tempdir().unwrap();
    let text = init_in(dir.path(), Audience::Personal, &["example.org"]);
    assert!(text.contains("begin: personal-agent-memory"));
    assert!(!text.contains("Team settings"));
}

#[test]
fn a5_team_has_the_team_skill_and_the_domains() {
    let dir = tempfile::tempdir().unwrap();
    let text = init_in(dir.path(), Audience::Team, &["example.org", "example.net"]);
    assert!(text.contains("begin: team-agent-memory"));
    assert!(text.contains("Allowed email domains for commits: example.org, example.net.\n"));
    let dir = tempfile::tempdir().unwrap();
    let text = init_in(dir.path(), Audience::Team, &[]);
    assert!(!text.contains("Team settings"));
}

#[test]
fn a10_git_archive_holds_the_skill_files() {
    let out = std::process::Command::new("git")
        .args(["archive", "--format=tar", "HEAD"])
        .current_dir(root())
        .output()
        .unwrap();
    if !out.status.success() {
        return; // not a Git checkout (for example a source tarball)
    }
    let list = std::process::Command::new("tar")
        .arg("-t")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut c| {
            use std::io::Write;
            c.stdin.take().unwrap().write_all(&out.stdout)?;
            c.wait_with_output()
        })
        .unwrap();
    let names = String::from_utf8_lossy(&list.stdout).to_string();
    // The files are committed with the spec. In a working tree that is not committed yet, skip.
    if !names.contains("plain-git/") {
        let tracked = std::process::Command::new("git")
            .args(["ls-files", "plain-git"])
            .current_dir(root())
            .output()
            .unwrap();
        assert!(
            String::from_utf8_lossy(&tracked.stdout).trim().is_empty(),
            "git archive leaves out plain-git/"
        );
        return;
    }
    assert!(names.contains("plain-git/personal-agent-memory/SKILL.md"));
    assert!(names.contains("plain-git/team-agent-memory/SKILL.md"));
}
