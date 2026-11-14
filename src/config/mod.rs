//! Limits and config layers (see `specs/foundation/trust-model`).
//!
//! The strictest layer wins. No tool reads a config file itself: every tool gets an `Effective` value.

pub mod layers;
pub mod limits;

pub use layers::{load_file_layers, parse_layer, read_layer, ConfigError, Paths};
pub use limits::{Access, ApprovalVia, Denied, Effective, Kind, Limits};

/// Merge the layers in order: the file layers, the shared layer from the knowledge base,
/// and the layer from the MCP entry.
pub fn effective(
    file_layers: &[Limits],
    shared: Option<&Limits>,
    entry: Option<&Limits>,
) -> Effective {
    let mut all: Vec<Limits> = file_layers.to_vec();
    all.extend(shared.cloned());
    all.extend(entry.cloned());
    Effective::from_layers(&all)
}
