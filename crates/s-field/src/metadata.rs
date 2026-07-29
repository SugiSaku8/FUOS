//! Dataset metadata.

use std::collections::BTreeMap;

/// Metadata stored in an SNC dataset.
///
/// This structure intentionally contains only descriptive
/// information. Numerical data is stored separately.
#[derive(Clone, Debug, Default)]
pub struct Metadata {
    dataset_name: String,
    title: String,
    institution: String,
    source: String,
    history: String,
    references: String,
    comment: String,

    attributes: BTreeMap<String, String>,
}

impl Metadata {
    /// Creates an empty metadata object.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    // -------------------------------------------------------------------------
    // Setters
    // -------------------------------------------------------------------------

    #[inline]
    pub fn set_dataset_name<S: Into<String>>(&mut self, value: S) {
        self.dataset_name = value.into();
    }

    #[inline]
    pub fn set_title<S: Into<String>>(&mut self, value: S) {
        self.title = value.into();
    }

    #[inline]
    pub fn set_institution<S: Into<String>>(&mut self, value: S) {
        self.institution = value.into();
    }

    #[inline]
    pub fn set_source<S: Into<String>>(&mut self, value: S) {
        self.source = value.into();
    }

    #[inline]
    pub fn set_history<S: Into<String>>(&mut self, value: S) {
        self.history = value.into();
    }

    #[inline]
    pub fn set_references<S: Into<String>>(&mut self, value: S) {
        self.references = value.into();
    }

    #[inline]
    pub fn set_comment<S: Into<String>>(&mut self, value: S) {
        self.comment = value.into();
    }

    /// Adds or replaces a custom attribute.
    #[inline]
    pub fn insert_attribute<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.attributes.insert(key.into(), value.into());
    }

    // -------------------------------------------------------------------------
    // Getters
    // -------------------------------------------------------------------------

    #[inline]
    pub fn dataset_name(&self) -> &str {
        &self.dataset_name
    }

    #[inline]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[inline]
    pub fn institution(&self) -> &str {
        &self.institution
    }

    #[inline]
    pub fn source(&self) -> &str {
        &self.source
    }

    #[inline]
    pub fn history(&self) -> &str {
        &self.history
    }

    #[inline]
    pub fn references(&self) -> &str {
        &self.references
    }

    #[inline]
    pub fn comment(&self) -> &str {
        &self.comment
    }

    #[inline]
    pub fn attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }

    #[inline]
    pub fn attributes(&self) -> &BTreeMap<String, String> {
        &self.attributes
    }

    #[inline]
    pub fn contains_attribute(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    #[inline]
    pub fn remove_attribute(&mut self, key: &str) -> Option<String> {
        self.attributes.remove(key)
    }

    #[inline]
    pub fn clear_attributes(&mut self) {
        self.attributes.clear();
    }
}
