#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("atom `{id}` not found")]
    NotFound { id: String },
    #[error("failed to parse atom `{id}`: {message}")]
    Parse { id: String, message: String },
    #[error("atom id mismatch: path `{path_id}` vs frontmatter `{atom_id}`")]
    IdMismatch { path_id: String, atom_id: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
