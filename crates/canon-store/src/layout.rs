use std::path::{Path, PathBuf};

pub fn namespace_dir(root: &Path) -> PathBuf {
    root.join("opencanon")
}

pub fn atoms_dir(root: &Path) -> PathBuf {
    namespace_dir(root).join("atoms")
}

pub fn atom_path(root: &Path, id: &str) -> PathBuf {
    atoms_dir(root).join(format!("{id}.md"))
}

pub fn docs_dir(root: &Path) -> PathBuf {
    namespace_dir(root).join("docs")
}

pub fn doc_path(root: &Path, id: &str) -> PathBuf {
    docs_dir(root).join(format!("{id}.md"))
}

pub fn atom_id_from_filename(name: &str) -> Option<String> {
    if name.starts_with('.') {
        return None;
    }
    name.strip_suffix(".md")
        .filter(|id| !id.is_empty())
        .map(ToOwned::to_owned)
}
