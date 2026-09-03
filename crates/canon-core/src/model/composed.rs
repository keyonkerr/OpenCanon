#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComposedDoc {
    pub id: String,
    pub title: String,
    pub atoms: Vec<String>,
    pub body: String,
}
