//! Which layer set the effective value of each limit (see `specs/features/maintenance`).

use crate::config::{Effective, Limits};
use serde_json::{json, Map, Value};

/// A named layer, in the order system, shared, personal, entry.
pub struct Layer<'a> {
    pub name: &'static str,
    pub limits: &'a Limits,
}

fn names(mut v: Vec<&'static str>) -> Value {
    v.dedup();
    if v.is_empty() {
        json!(["default"])
    } else {
        json!(v)
    }
}

fn scalar<T: PartialEq>(
    layers: &[Layer],
    effective: &T,
    get: impl Fn(&Limits) -> Option<T>,
) -> Value {
    names(
        layers
            .iter()
            .filter(|l| get(l.limits).as_ref() == Some(effective))
            .map(|l| l.name)
            .collect(),
    )
}

fn list<T>(layers: &[Layer], get: impl Fn(&Limits) -> Option<&T>) -> Value {
    names(
        layers
            .iter()
            .filter(|l| get(l.limits).is_some())
            .map(|l| l.name)
            .collect(),
    )
}

/// The layers that set the effective value of each limit. A limit that no layer sets has
/// `["default"]`. A list limit names every layer that adds to the union.
pub fn limit_sources(layers: &[Layer], effective: &Effective) -> Value {
    let mut m = Map::new();
    m.insert(
        "access".into(),
        scalar(layers, &effective.access, |l| l.access),
    );
    m.insert(
        "paused".into(),
        scalar(layers, &effective.paused, |l| l.paused),
    );
    m.insert(
        "tools_denied".into(),
        list(layers, |l| l.tools_denied.as_ref()),
    );
    m.insert("folders".into(), list(layers, |l| l.folders.as_ref()));
    m.insert(
        "max_results".into(),
        scalar(layers, &effective.max_results, |l| l.max_results),
    );
    m.insert(
        "max_tokens".into(),
        scalar(layers, &effective.max_tokens, |l| l.max_tokens),
    );
    m.insert(
        "max_documents_opened".into(),
        scalar(layers, &effective.max_documents_opened, |l| {
            l.max_documents_opened
        }),
    );
    m.insert(
        "allowed_repo_hosts".into(),
        list(layers, |l| l.allowed_repo_hosts.as_ref()),
    );
    m.insert(
        "approval_required_for".into(),
        list(layers, |l| l.approval_required_for.as_ref()),
    );
    m.insert(
        "approval_via".into(),
        scalar(layers, &effective.approval_via, |l| l.approval_via),
    );
    Value::Object(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalars_name_the_layer_of_the_effective_value_and_lists_name_all() {
        let shared = Limits {
            max_results: Some(3),
            tools_denied: Some(vec!["delete".into()]),
            ..Limits::default()
        };
        let personal = Limits {
            max_results: Some(5),
            tools_denied: Some(vec!["move".into()]),
            ..Limits::default()
        };
        let layers = [
            Layer {
                name: "shared",
                limits: &shared,
            },
            Layer {
                name: "personal",
                limits: &personal,
            },
        ];
        let eff = Effective::from_layers(&[shared.clone(), personal.clone()]);
        let s = limit_sources(&layers, &eff);
        assert_eq!(s["max_results"], json!(["shared"]));
        assert_eq!(s["tools_denied"], json!(["shared", "personal"]));
        assert_eq!(s["max_tokens"], json!(["default"]));
        assert_eq!(s["access"], json!(["default"]));
    }
}
