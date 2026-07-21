#[derive(Clone, Debug)]
pub struct Metadata {
    pub title: String,

    pub source: String,

    pub institution: String,

    pub history: String,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            title: String::new(),

            source: String::new(),

            institution: String::new(),

            history: String::new(),
        }
    }
}
