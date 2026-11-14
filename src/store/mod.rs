//! The documents of a memory repository, loaded from the working tree for one call.
//!
//! `search-and-read`, `quality-checks`, `updates`, and others share this loader.

pub mod backlinks;
pub mod changes;
pub mod rank;
pub mod scope;
pub mod stats;

use crate::format;
use crate::format::frontmatter::Frontmatter;
use crate::memrepo::MemRepo;

pub use scope::Scope;

pub struct Doc {
    pub id: String,
    pub raw: String,
    /// `None` if the file cannot be parsed.
    pub front: Option<Frontmatter>,
    pub body: String,
}

impl Doc {
    pub fn parsed(&self) -> bool {
        self.front.is_some()
    }

    pub fn text(&self, key: &str) -> Option<&str> {
        self.front.as_ref()?.get_text(key)
    }

    pub fn title(&self) -> &str {
        self.text("title").unwrap_or("")
    }

    pub fn description(&self) -> &str {
        self.text("description").unwrap_or("")
    }

    pub fn status(&self) -> &str {
        self.text("status").unwrap_or("")
    }

    pub fn updated(&self) -> &str {
        self.text("updated").unwrap_or("")
    }

    pub fn list(&self, key: &str) -> Vec<String> {
        self.front
            .as_ref()
            .and_then(|f| f.get_list(key))
            .map(<[String]>::to_vec)
            .unwrap_or_default()
    }

    pub fn tags(&self) -> Vec<String> {
        self.list("tags")
    }

    pub fn disputed(&self) -> bool {
        self.front.as_ref().and_then(|f| f.get_bool("disputed")) == Some(true)
    }

    /// A generated index file: named `index` and tagged `index`.
    pub fn is_index(&self) -> bool {
        (self.id == "index" || self.id.ends_with("/index"))
            && self.tags().iter().any(|t| t == "index")
    }

    /// The IDs in a property list of wikilinks, such as `related`.
    pub fn property_ids(&self, key: &str) -> Vec<String> {
        self.list(key)
            .iter()
            .filter_map(|i| format::fields::property_link_target(i).map(str::to_string))
            .collect()
    }

    /// Words of title, description, and body (section 9.1 of the standard).
    pub fn words(&self) -> usize {
        format::count::words(self.title())
            + format::count::words(self.description())
            + format::count::body_words(&self.body)
    }
}

pub struct Store {
    docs: Vec<Doc>,
}

impl Store {
    /// Load every document. A file that cannot be read as text is skipped.
    pub fn load(memrepo: &MemRepo) -> Store {
        let docs = memrepo
            .documents()
            .into_iter()
            .filter_map(|d| {
                let raw = std::fs::read(memrepo.path_of(&d.id).ok()?).ok()?;
                let raw = String::from_utf8(raw).ok()?;
                Some(match format::parse(&raw) {
                    Ok(doc) => Doc {
                        id: d.id,
                        raw,
                        front: Some(doc.frontmatter),
                        body: doc.body,
                    },
                    Err(_) => Doc {
                        id: d.id,
                        raw,
                        front: None,
                        body: String::new(),
                    },
                })
            })
            .collect();
        Store { docs }
    }

    /// The documents that read tools use: parsed, and not generated index files.
    pub fn docs(&self) -> impl Iterator<Item = &Doc> {
        self.docs.iter().filter(|d| d.parsed() && !d.is_index())
    }

    /// The parsed documents including generated index files.
    pub fn docs_with_index(&self) -> impl Iterator<Item = &Doc> {
        self.docs.iter().filter(|d| d.parsed())
    }

    /// Every document, also those that cannot be parsed and index files.
    pub fn all_docs(&self) -> &[Doc] {
        &self.docs
    }

    pub fn get(&self, id: &str) -> Option<&Doc> {
        self.docs.iter().find(|d| d.id == id)
    }
}
