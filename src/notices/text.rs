//! The texts of the notices (see `specs/features/notifications-and-review`).

pub const CLONE_BEHIND: &str = "The last update from the remote failed. The clone may be behind.";
pub const PAUSED: &str = "Writes are paused.";

/// `a`, `b`, `c` and 2 more. At most 3 IDs are named.
pub fn name_ids(ids: &[String]) -> String {
    let shown: Vec<String> = ids.iter().take(3).map(|i| format!("`{i}`")).collect();
    let mut text = shown.join(", ");
    if ids.len() > 3 {
        text.push_str(&format!(" and {} more", ids.len() - 3));
    }
    text
}

pub fn disputed(ids: &[String]) -> String {
    format!(
        "Someone disputed {} of your documents: {}. Run `updates` for details.",
        ids.len(),
        name_ids(ids)
    )
}

pub fn changed(ids: &[String]) -> String {
    format!(
        "Others changed {} of your documents: {}. Run `updates` for details.",
        ids.len(),
        name_ids(ids)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_most_three_ids_are_named() {
        let ids: Vec<String> = (1..=5).map(|i| format!("ops/d{i}")).collect();
        let t = disputed(&ids);
        assert!(t.contains("`ops/d1`, `ops/d2`, `ops/d3` and 2 more"), "{t}");
        assert!(!t.contains("d4"));
        assert_eq!(
            changed(&ids[..1]),
            "Others changed 1 of your documents: `ops/d1`. Run `updates` for details."
        );
    }
}
