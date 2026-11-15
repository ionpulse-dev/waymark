//! Texts that depend on the system and on how waymark was installed.
//!
//! The advice must fit the person. A Linux binary can come from Homebrew, and a macOS binary can be
//! copied by hand, so the install path decides the update hint, and the operating system decides the
//! Git hint. Both are decided at run time. See `specs/foundation/linux-support`.

use std::path::Path;

/// Folder names that show a Homebrew install (macOS and Linux).
const HOMEBREW_MARKERS: [&str; 3] = ["/Cellar/", "/homebrew/", "/linuxbrew/"];

/// True when the path of the binary is inside a Homebrew folder.
pub fn is_homebrew_path(path: &Path) -> bool {
    let p = path.to_string_lossy();
    HOMEBREW_MARKERS.iter().any(|m| p.contains(m))
}

/// True when the running binary was installed with Homebrew.
pub fn installed_with_homebrew() -> bool {
    std::env::current_exe()
        .map(|p| is_homebrew_path(&p))
        .unwrap_or(false)
}

/// The step that updates waymark, as an imperative that continues a sentence
/// ("..., run `brew upgrade waymark` and restart ...").
pub fn update_step_for(homebrew: bool) -> &'static str {
    if homebrew {
        "run `brew upgrade waymark`"
    } else {
        "update waymark the way you installed it"
    }
}

/// The update step for the running binary.
pub fn update_step() -> &'static str {
    update_step_for(installed_with_homebrew())
}

/// The update step as its own sentence, without the final period.
pub fn update_sentence_for(homebrew: bool) -> &'static str {
    if homebrew {
        "Update waymark with `brew upgrade waymark`"
    } else {
        "Update waymark the way you installed it"
    }
}

/// The update sentence for the running binary.
pub fn update_sentence() -> &'static str {
    update_sentence_for(installed_with_homebrew())
}

/// What to tell a person whose `git` is missing or does not work. `os` is a value of
/// `std::env::consts::OS`.
pub fn git_install_hint_for(os: &str) -> &'static str {
    if os == "macos" {
        "Install the Xcode command line tools: run `xcode-select --install`, or install Git with Homebrew."
    } else {
        "Install Git with the package manager of your system, for example `apt install git`."
    }
}

/// The Git install hint for the system that waymark runs on.
pub fn git_install_hint() -> &'static str {
    git_install_hint_for(std::env::consts::OS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn homebrew_folders_are_found() {
        for p in [
            "/opt/homebrew/Cellar/waymark/0.4.0/bin/waymark",
            "/opt/homebrew/bin/waymark",
            "/usr/local/Homebrew/bin/x/homebrew/waymark",
            "/home/linuxbrew/.linuxbrew/bin/waymark",
        ] {
            assert!(is_homebrew_path(&PathBuf::from(p)), "{p}");
        }
    }

    #[test]
    fn other_folders_are_not_homebrew() {
        for p in [
            "/usr/local/bin/waymark",
            "/home/me/.local/bin/waymark",
            "/work/waymark/target/debug/waymark",
        ] {
            assert!(!is_homebrew_path(&PathBuf::from(p)), "{p}");
        }
    }

    #[test]
    fn the_update_hint_says_brew_only_for_homebrew() {
        assert!(update_step_for(true).contains("brew upgrade waymark"));
        assert!(update_sentence_for(true).contains("brew upgrade waymark"));
        for s in [update_step_for(false), update_sentence_for(false)] {
            assert!(!s.contains("brew"), "{s}");
            assert!(s.contains("the way you installed it"), "{s}");
        }
    }

    #[test]
    fn the_git_hint_fits_the_system() {
        let mac = git_install_hint_for("macos");
        assert!(mac.contains("xcode-select --install"), "{mac}");
        let linux = git_install_hint_for("linux");
        assert!(linux.contains("package manager"), "{linux}");
        assert!(!linux.contains("xcode-select"), "{linux}");
    }

    #[test]
    fn the_running_system_gets_a_hint() {
        assert!(!git_install_hint().is_empty());
        assert!(!update_step().is_empty());
        assert!(!update_sentence().is_empty());
    }
}
