//! Limits and the rule "the strictest layer wins" (see `trust-model`).

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Access {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalVia {
    Elicitation,
    Console,
}

/// What a tool does. Read tools are always allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Read,
    Write,
    Destructive,
}

/// The limits that one layer sets. `None` means the layer says nothing.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Limits {
    pub access: Option<Access>,
    pub paused: Option<bool>,
    pub tools_denied: Option<Vec<String>>,
    pub folders: Option<Vec<String>>,
    pub max_results: Option<usize>,
    pub max_tokens: Option<usize>,
    pub max_documents_opened: Option<usize>,
    pub allowed_repo_hosts: Option<Vec<String>>,
    pub approval_required_for: Option<Vec<String>>,
    pub approval_via: Option<ApprovalVia>,
}

/// Why a tool call is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Denied {
    /// A limit blocks the call: read-only access, a switched-off tool, or a folder.
    AccessDenied(String),
    Paused,
}

/// Tools that got a new name, as (old name, new name) (see `tool-conventions`).
pub const RENAMED_TOOLS: [(&str, &str); 9] = [
    ("add", "add_document"),
    ("delete", "delete_document"),
    ("deprecate", "deprecate_document"),
    ("dispute", "dispute_document"),
    ("verify", "verify_document"),
    ("reject", "reject_document"),
    ("add_empty_folder", "create_folder"),
    ("remove_folder", "delete_folder"),
    ("folders", "folder_tree"),
];

/// The current name of a tool. An old name gives the new name. Any other name stays as it is.
pub fn canonical_tool(name: &str) -> &str {
    RENAMED_TOOLS
        .iter()
        .find(|(old, _)| *old == name)
        .map_or(name, |(_, new)| new)
}

pub const DEFAULT_MAX_RESULTS: usize = 5;
pub const DEFAULT_MAX_TOKENS: usize = 800;
pub const DEFAULT_MAX_DOCUMENTS_OPENED: usize = 20;

/// The limits that apply to a call, after all layers are merged.
#[derive(Debug, Clone)]
pub struct Effective {
    pub access: Access,
    pub paused: bool,
    pub tools_denied: BTreeSet<String>,
    /// One list of globs for each layer that sets folders. A path must match every list.
    pub folder_layers: Vec<Vec<String>>,
    pub max_results: usize,
    pub max_tokens: usize,
    pub max_documents_opened: usize,
    /// One list of hosts for each layer that sets hosts. A host must be in every list.
    pub host_layers: Vec<Vec<String>>,
    pub approval_required_for: BTreeSet<String>,
    pub approval_via: ApprovalVia,
    folder_sets: Vec<GlobSet>,
}

impl Default for Effective {
    fn default() -> Self {
        Effective::from_layers(&[])
    }
}

/// The patterns of one layer. A plain folder name such as `engineering` means `engineering/**`.
/// A pattern `a/**` also matches the folder `a` itself.
fn compile_layer(patterns: &[String]) -> GlobSet {
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        let q = p.trim_matches('/');
        let mut add = |pattern: &str| {
            if let Ok(g) = Glob::new(pattern) {
                b.add(g);
            }
        };
        if q.contains(['*', '?', '[', '{']) {
            add(q);
            if let Some(base) = q.strip_suffix("/**") {
                add(base);
            }
        } else {
            add(q);
            add(&format!("{q}/**"));
        }
    }
    b.build()
        .unwrap_or_else(|_| GlobSetBuilder::new().build().expect("empty glob set"))
}

impl Effective {
    /// Merge layers. The strictest value of each limit wins. A layer can tighten a limit and never loosen it.
    pub fn from_layers(layers: &[Limits]) -> Effective {
        let mut access = Access::ReadWrite;
        let mut paused = false;
        let mut tools_denied = BTreeSet::new();
        let mut folder_layers: Vec<Vec<String>> = Vec::new();
        let mut host_layers: Vec<Vec<String>> = Vec::new();
        let mut max_results = DEFAULT_MAX_RESULTS;
        let mut max_tokens = DEFAULT_MAX_TOKENS;
        let mut max_opened = DEFAULT_MAX_DOCUMENTS_OPENED;
        let mut approval = BTreeSet::new();
        let mut via = ApprovalVia::Elicitation;
        for l in layers {
            if l.access == Some(Access::ReadOnly) {
                access = Access::ReadOnly;
            }
            if l.paused == Some(true) {
                paused = true;
            }
            if let Some(t) = &l.tools_denied {
                // An old tool name in a limit counts as the new name: a rename never loosens a limit.
                tools_denied.extend(t.iter().map(|n| canonical_tool(n).to_string()));
            }
            if let Some(f) = &l.folders {
                if !f.is_empty() && !folder_layers.contains(f) {
                    folder_layers.push(f.clone());
                }
            }
            if let Some(h) = &l.allowed_repo_hosts {
                if !h.is_empty() && !host_layers.contains(h) {
                    host_layers.push(h.clone());
                }
            }
            if let Some(n) = l.max_results {
                max_results = max_results.min(n);
            }
            if let Some(n) = l.max_tokens {
                max_tokens = max_tokens.min(n);
            }
            if let Some(n) = l.max_documents_opened {
                max_opened = max_opened.min(n);
            }
            if let Some(a) = &l.approval_required_for {
                approval.extend(a.iter().map(|n| canonical_tool(n).to_string()));
            }
            if l.approval_via == Some(ApprovalVia::Console) {
                via = ApprovalVia::Console;
            }
        }
        folder_layers.sort();
        host_layers.sort();
        let folder_sets = folder_layers
            .iter()
            .map(|layer| compile_layer(layer))
            .collect();
        Effective {
            access,
            paused,
            tools_denied,
            folder_layers,
            max_results,
            max_tokens,
            max_documents_opened: max_opened,
            host_layers,
            approval_required_for: approval,
            approval_via: via,
            folder_sets,
        }
    }

    /// Is a document (by ID) inside the allowed folders? No folder limit means all.
    pub fn allows_folder(&self, id: &str) -> bool {
        self.folder_sets
            .iter()
            .all(|set| set.is_match(id.trim_matches('/')))
    }

    /// Is a Git host allowed? No host limit means any.
    pub fn allows_host(&self, host: &str) -> bool {
        self.host_layers
            .iter()
            .all(|l| l.iter().any(|h| h.eq_ignore_ascii_case(host)))
    }

    /// Can a tool run? Reads are always allowed, except a tool that is switched off.
    pub fn check_tool(&self, name: &str, kind: Kind) -> Result<(), Denied> {
        if self.tools_denied.contains(name) {
            return Err(Denied::AccessDenied(format!(
                "The tool \"{name}\" is switched off."
            )));
        }
        if kind != Kind::Read {
            // Read-only comes first: a resume would not help (see `pause-and-resume`).
            if self.access == Access::ReadOnly {
                return Err(Denied::AccessDenied(
                    "Writing is not allowed: access is read-only.".to_string(),
                ));
            }
            if self.paused {
                return Err(Denied::Paused);
            }
        }
        Ok(())
    }

    /// Is a tool listed to the client?
    pub fn lists_tool(&self, name: &str, kind: Kind) -> bool {
        self.check_tool(name, kind).is_ok()
    }

    /// Does this tool need approval before it runs?
    pub fn needs_approval(&self, name: &str, kind: Kind) -> bool {
        self.approval_required_for.contains(name)
            || (kind != Kind::Read && self.approval_required_for.contains("write"))
    }

    /// The `settings` field of every result.
    pub fn to_json(&self) -> Value {
        let folders = match self.folder_layers.len() {
            0 => json!([]),
            1 => json!(self.folder_layers[0]),
            _ => json!(self.folder_layers),
        };
        json!({
            "access": match self.access { Access::ReadOnly => "read-only", Access::ReadWrite => "read-write" },
            "paused": self.paused,
            "tools_denied": self.tools_denied,
            "folders": folders,
            "max_results": self.max_results,
            "max_tokens": self.max_tokens,
            "max_documents_opened": self.max_documents_opened,
            "approval": {
                "required_for": self.approval_required_for,
                "via": match self.approval_via { ApprovalVia::Elicitation => "elicitation", ApprovalVia::Console => "console" },
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(f: impl FnOnce(&mut Limits)) -> Limits {
        let mut x = Limits::default();
        f(&mut x);
        x
    }

    #[test]
    fn defaults_apply_when_no_layer_sets_anything() {
        let e = Effective::from_layers(&[]);
        assert_eq!(e.access, Access::ReadWrite);
        assert!(!e.paused);
        assert_eq!(e.max_results, 5);
        assert_eq!(e.max_tokens, 800);
        assert_eq!(e.max_documents_opened, 20);
        assert!(e.allows_folder("any/where"));
        assert!(e.allows_host("example.org"));
        assert!(e.tools_denied.is_empty());
    }

    #[test]
    fn read_only_wins_over_read_write() {
        let e = Effective::from_layers(&[
            l(|x| x.access = Some(Access::ReadWrite)),
            l(|x| x.access = Some(Access::ReadOnly)),
        ]);
        assert_eq!(e.access, Access::ReadOnly);
        assert!(e.check_tool("add_document", Kind::Write).is_err());
        assert!(e.check_tool("search", Kind::Read).is_ok());
    }

    #[test]
    fn numeric_caps_take_the_smallest_value() {
        let e = Effective::from_layers(&[
            l(|x| x.max_results = Some(5)),
            l(|x| x.max_results = Some(3)),
        ]);
        assert_eq!(e.max_results, 3);
        // A layer cannot raise a cap above the built-in default either.
        let e = Effective::from_layers(&[l(|x| x.max_results = Some(50))]);
        assert_eq!(e.max_results, 5);
    }

    #[test]
    fn paused_if_any_layer_says_so() {
        let e =
            Effective::from_layers(&[l(|x| x.paused = Some(false)), l(|x| x.paused = Some(true))]);
        assert_eq!(
            e.check_tool("add_document", Kind::Write),
            Err(Denied::Paused)
        );
        assert!(e.check_tool("show", Kind::Read).is_ok());
    }

    #[test]
    fn denied_tools_are_the_union_and_are_not_listed() {
        let e = Effective::from_layers(&[
            l(|x| x.tools_denied = Some(vec!["delete_document".into()])),
            l(|x| x.tools_denied = Some(vec!["move".into()])),
        ]);
        assert!(!e.lists_tool("delete_document", Kind::Destructive));
        assert!(!e.lists_tool("move", Kind::Write));
        assert!(e.lists_tool("add_document", Kind::Write));
        assert!(matches!(
            e.check_tool("delete_document", Kind::Destructive),
            Err(Denied::AccessDenied(_))
        ));
    }

    #[test]
    fn an_old_tool_name_in_a_limit_counts_as_the_new_name() {
        let e = Effective::from_layers(&[
            l(|x| x.tools_denied = Some(vec!["delete".into(), "folders".into()])),
            l(|x| x.approval_required_for = Some(vec!["add".into(), "write".into()])),
        ]);
        assert!(!e.lists_tool("delete_document", Kind::Destructive));
        assert!(!e.lists_tool("folder_tree", Kind::Read));
        assert!(e.tools_denied.contains("delete_document"));
        assert!(e.approval_required_for.contains("add_document"));
        assert!(e.approval_required_for.contains("write"));
        assert!(e.lists_tool("add_document", Kind::Write));
    }

    #[test]
    fn canonical_tool_maps_every_old_name_and_leaves_other_names_alone() {
        for (old, new) in RENAMED_TOOLS {
            assert_eq!(canonical_tool(old), new);
            assert_eq!(canonical_tool(new), new);
        }
        assert_eq!(canonical_tool("move"), "move");
        assert_eq!(canonical_tool("search"), "search");
        assert_eq!(canonical_tool("write"), "write");
    }

    #[test]
    fn a_read_tool_can_be_switched_off() {
        let e = Effective::from_layers(&[l(|x| x.tools_denied = Some(vec!["stats".into()]))]);
        assert!(!e.lists_tool("stats", Kind::Read));
    }

    #[test]
    fn folders_must_match_every_layer() {
        let e = Effective::from_layers(&[
            l(|x| x.folders = Some(vec!["engineering/**".into()])),
            l(|x| x.folders = Some(vec!["engineering/backend".into()])),
        ]);
        assert!(e.allows_folder("engineering/backend/a"));
        assert!(!e.allows_folder("engineering/frontend/a"));
        assert!(!e.allows_folder("ops/x"));
    }

    #[test]
    fn folder_globs() {
        let e = Effective::from_layers(&[l(|x| x.folders = Some(vec!["engineering/**".into()]))]);
        assert!(e.allows_folder("engineering/a/b"));
        assert!(e.allows_folder("engineering"));
        assert!(!e.allows_folder("ops/x"));
        assert!(!e.allows_folder("engineering-x/a"));
    }

    #[test]
    fn an_empty_folder_list_means_all() {
        let e = Effective::from_layers(&[l(|x| x.folders = Some(vec![]))]);
        assert!(e.allows_folder("ops/x"));
    }

    #[test]
    fn hosts_must_be_in_every_list() {
        let e =
            Effective::from_layers(&[
                l(|x| {
                    x.allowed_repo_hosts = Some(vec!["github.com".into(), "git.example.org".into()])
                }),
                l(|x| x.allowed_repo_hosts = Some(vec!["github.com".into()])),
            ]);
        assert!(e.allows_host("github.com"));
        assert!(!e.allows_host("git.example.org"));
    }

    #[test]
    fn approval_is_the_union_and_console_wins() {
        let e = Effective::from_layers(&[
            l(|x| x.approval_required_for = Some(vec!["delete_document".into()])),
            l(|x| {
                x.approval_required_for = Some(vec!["write".into()]);
                x.approval_via = Some(ApprovalVia::Console);
            }),
        ]);
        assert!(e.needs_approval("delete_document", Kind::Destructive));
        assert!(e.needs_approval("add_document", Kind::Write));
        assert!(!e.needs_approval("search", Kind::Read));
        assert_eq!(e.approval_via, ApprovalVia::Console);
    }

    #[test]
    fn json_shows_the_effective_limits() {
        let j = Effective::default().to_json();
        assert_eq!(j["access"], "read-write");
        assert_eq!(j["max_results"], 5);
        assert_eq!(j["approval"]["via"], "elicitation");
    }

    /// Small deterministic generator for the property tests.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 33
        }
    }

    fn random_layer(r: &mut Lcg) -> Limits {
        let tools = ["add_document", "delete_document", "move", "stats"];
        Limits {
            access: match r.next() % 3 {
                0 => None,
                1 => Some(Access::ReadOnly),
                _ => Some(Access::ReadWrite),
            },
            paused: match r.next() % 3 {
                0 => None,
                1 => Some(true),
                _ => Some(false),
            },
            tools_denied: if r.next().is_multiple_of(2) {
                None
            } else {
                Some(vec![tools[(r.next() % 4) as usize].to_string()])
            },
            folders: None,
            max_results: if r.next().is_multiple_of(2) {
                None
            } else {
                Some((r.next() % 9) as usize)
            },
            max_tokens: if r.next().is_multiple_of(2) {
                None
            } else {
                Some((r.next() % 2000) as usize)
            },
            max_documents_opened: None,
            allowed_repo_hosts: None,
            approval_required_for: if r.next().is_multiple_of(2) {
                None
            } else {
                Some(vec![tools[(r.next() % 4) as usize].to_string()])
            },
            approval_via: match r.next() % 3 {
                0 => None,
                1 => Some(ApprovalVia::Console),
                _ => Some(ApprovalVia::Elicitation),
            },
        }
    }

    fn same(a: &Effective, b: &Effective) -> bool {
        a.to_json() == b.to_json()
    }

    #[test]
    fn merge_is_order_independent_idempotent_and_never_looser() {
        let mut r = Lcg(7);
        for _ in 0..500 {
            let layers: Vec<Limits> = (0..4).map(|_| random_layer(&mut r)).collect();
            let all = Effective::from_layers(&layers);
            let mut rev = layers.clone();
            rev.reverse();
            assert!(same(&all, &Effective::from_layers(&rev)), "order");
            let mut twice = layers.clone();
            twice.extend(layers.clone());
            assert!(same(&all, &Effective::from_layers(&twice)), "idempotent");
            // Never looser than any single layer.
            for one in &layers {
                let single = Effective::from_layers(std::slice::from_ref(one));
                assert!(all.max_results <= single.max_results);
                assert!(all.max_tokens <= single.max_tokens);
                assert!(all.tools_denied.is_superset(&single.tools_denied));
                assert!(all
                    .approval_required_for
                    .is_superset(&single.approval_required_for));
                if single.access == Access::ReadOnly {
                    assert_eq!(all.access, Access::ReadOnly);
                }
                if single.paused {
                    assert!(all.paused);
                }
                if single.approval_via == ApprovalVia::Console {
                    assert_eq!(all.approval_via, ApprovalVia::Console);
                }
            }
        }
    }
}
