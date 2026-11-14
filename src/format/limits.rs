//! Size limits of a document (section 9 of the standard).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub title_max_words: usize,
    pub description_max_words: usize,
    pub max_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            title_max_words: 15,
            description_max_words: 50,
            max_bytes: 102_400,
        }
    }
}
