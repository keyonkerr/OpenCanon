use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use canon_core::{Atom, ComposedDoc};

use crate::layout::{
    atom_id_from_filename, atom_path, atoms_dir, config_path, doc_path, docs_dir, namespace_dir,
};
use crate::serialize;
use crate::serialize_config;
use crate::serialize_doc;
use crate::Error;

pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn write(&self, atom: &Atom) -> Result<(), Error> {
        let dir = atoms_dir(&self.root);
        fs::create_dir_all(&dir)?;
        let dest = atom_path(&self.root, &atom.id);
        let tmp = dir.join(format!(".{}.md.tmp", atom.id));
        let rendered = serialize::to_markdown(atom);

        {
            let mut file = File::create(&tmp)?;
            file.write_all(rendered.as_bytes())?;
            file.sync_all()?;
        }

        replace_file(&tmp, &dest)?;
        Ok(())
    }

    pub fn read(&self, id: &str) -> Result<Atom, Error> {
        let path = atom_path(&self.root, id);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(Error::NotFound { id: id.to_string() });
            }
            Err(e) => return Err(e.into()),
        };
        let atom = serialize::from_markdown(id, &text)?;
        if atom.id != id {
            return Err(Error::IdMismatch {
                path_id: id.to_string(),
                atom_id: atom.id,
            });
        }
        Ok(atom)
    }

    pub fn delete(&self, id: &str) -> Result<(), Error> {
        let path = atom_path(&self.root, id);
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Err(Error::NotFound { id: id.to_string() })
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Occupied ids from filenames. Missing `opencanon/atoms/` is an empty set.
    pub fn list_ids(&self) -> Result<HashSet<String>, Error> {
        let dir = atoms_dir(&self.root);
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(HashSet::new()),
            Err(e) => return Err(e.into()),
        };
        let mut ids = HashSet::new();
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if let Some(id) = atom_id_from_filename(name) {
                ids.insert(id);
            }
        }
        Ok(ids)
    }

    /// All atoms on disk. Missing directory is empty; does not create it.
    pub fn list(&self) -> Result<Vec<Atom>, Error> {
        let mut ids: Vec<String> = self.list_ids()?.into_iter().collect();
        ids.sort();
        let mut atoms = Vec::with_capacity(ids.len());
        for id in ids {
            atoms.push(self.read(&id)?);
        }
        Ok(atoms)
    }

    pub fn init_namespace(&self) -> Result<(), Error> {
        fs::create_dir_all(atoms_dir(&self.root))?;
        Ok(())
    }

    pub fn write_config(&self, locales: &[String]) -> Result<(), Error> {
        let dir = namespace_dir(&self.root);
        fs::create_dir_all(&dir)?;
        let dest = config_path(&self.root);
        let tmp = dir.join(".config.yaml.tmp");
        let rendered = serialize_config::to_yaml(locales);

        {
            let mut file = File::create(&tmp)?;
            file.write_all(rendered.as_bytes())?;
            file.sync_all()?;
        }

        replace_file(&tmp, &dest)?;
        Ok(())
    }

    pub fn write_doc(&self, doc: &ComposedDoc) -> Result<(), Error> {
        let dir = docs_dir(&self.root);
        fs::create_dir_all(&dir)?;
        let dest = doc_path(&self.root, &doc.id);
        let tmp = dir.join(format!(".{}.md.tmp", doc.id));
        let rendered = serialize_doc::to_markdown(doc);

        {
            let mut file = File::create(&tmp)?;
            file.write_all(rendered.as_bytes())?;
            file.sync_all()?;
        }

        replace_file(&tmp, &dest)?;
        Ok(())
    }

    pub fn read_doc(&self, id: &str) -> Result<ComposedDoc, Error> {
        let path = doc_path(&self.root, id);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(Error::NotFound { id: id.to_string() });
            }
            Err(e) => return Err(e.into()),
        };
        let doc = serialize_doc::from_markdown(id, &text)?;
        if doc.id != id {
            return Err(Error::IdMismatch {
                path_id: id.to_string(),
                atom_id: doc.id,
            });
        }
        Ok(doc)
    }
}

fn replace_file(tmp: &Path, dest: &Path) -> Result<(), Error> {
    if dest.exists() {
        let bak = dest.with_extension("md.bak");
        fs::rename(dest, &bak)?;
        if let Err(e) = fs::rename(tmp, dest) {
            let _ = fs::rename(&bak, dest);
            let _ = fs::remove_file(tmp);
            return Err(e.into());
        }
        let _ = fs::remove_file(&bak);
    } else if let Err(e) = fs::rename(tmp, dest) {
        let _ = fs::remove_file(tmp);
        return Err(e.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use canon_core::{Atom, Freshness, Status};

    use super::Store;
    use crate::layout::{atoms_dir, config_path, docs_dir, namespace_dir};

    fn sample(id: &str, status: Status) -> Atom {
        Atom {
            id: id.into(),
            status,
            title: "t".into(),
            tags: vec![],
            freshness: Freshness::default(),
            body: "b".into(),
        }
    }

    #[test]
    fn list_missing_dir_is_empty_and_creates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path());
        assert!(store.list().unwrap().is_empty());
        assert!(store.list_ids().unwrap().is_empty());
        assert!(!namespace_dir(dir.path()).exists());
        assert!(!atoms_dir(dir.path()).exists());
    }

    #[test]
    fn write_creates_namespace_and_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path());
        let atom = Atom {
            id: "durability_daily_restore".into(),
            status: Status::Draft,
            title: "禁军突围装备耐久恢复机制".into(),
            tags: vec!["armybreak".into(), "durability".into()],
            freshness: Freshness {
                impl_path: "gamesvr/DurabilityManager.java".into(),
                ..Freshness::default()
            },
            body: "正文：只描述一个事实。".into(),
        };
        store.write(&atom).unwrap();
        assert!(atoms_dir(dir.path()).is_dir());
        assert!(!dir.path().join("opencanon").join("topics").exists());
        assert!(!dir.path().join("opencanon").join("docs").exists());
        let raw =
            std::fs::read_to_string(atoms_dir(dir.path()).join(format!("{}.md", atom.id))).unwrap();
        assert!(raw.starts_with("---\nid: "));
        assert_eq!(store.read(&atom.id).unwrap(), atom);
    }

    #[test]
    fn overwrite_and_delete_leave_siblings() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path());
        let a = sample("atom_a", Status::Draft);
        let b = sample("atom_b", Status::Active);
        store.write(&a).unwrap();
        store.write(&b).unwrap();

        let mut updated = a.clone();
        updated.title = "new".into();
        store.write(&updated).unwrap();
        assert_eq!(store.read(&a.id).unwrap().title, "new");
        assert_eq!(store.read(&b.id).unwrap().body, "b");

        store.delete(&a.id).unwrap();
        match store.read(&a.id) {
            Err(crate::Error::NotFound { id }) => assert_eq!(id, a.id),
            other => panic!("unexpected {other:?}"),
        }
        assert!(store.read(&b.id).is_ok());
    }

    #[test]
    fn delete_missing_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path());
        match store.delete("missing") {
            Err(crate::Error::NotFound { id }) => assert_eq!(id, "missing"),
            other => panic!("unexpected {other:?}"),
        }
        assert!(!namespace_dir(dir.path()).exists());
    }

    #[test]
    fn init_namespace_creates_atoms_not_docs() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path());
        store.init_namespace().unwrap();
        assert!(atoms_dir(dir.path()).is_dir());
        assert!(!docs_dir(dir.path()).exists());
    }

    #[test]
    fn write_config_roundtrips_locales_and_skips_docs() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path());
        store.write_config(&["en".into(), "ja".into()]).unwrap();
        let raw = std::fs::read_to_string(config_path(dir.path())).unwrap();
        assert_eq!(raw, "locales:\n  - en\n  - ja\n");
        assert!(!atoms_dir(dir.path()).exists());
        assert!(!docs_dir(dir.path()).exists());

        store.write_config(&[]).unwrap();
        let raw = std::fs::read_to_string(config_path(dir.path())).unwrap();
        assert_eq!(raw, "{}\n");
    }

    #[test]
    fn write_doc_creates_docs_dir_roundtrips_and_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path());
        let doc = canon_core::ComposedDoc {
            id: "how_ssot_works".into(),
            title: "t".into(),
            atoms: vec!["ssot_one_place".into()],
            body: "# t\n\nx [ssot_one_place](../atoms/ssot_one_place.md)".into(),
        };
        store.write_doc(&doc).unwrap();
        assert!(crate::layout::docs_dir(dir.path()).is_dir());
        assert!(!atoms_dir(dir.path()).exists());
        assert_eq!(store.read_doc(&doc.id).unwrap(), doc);

        let mut updated = doc.clone();
        updated.title = "new".into();
        store.write_doc(&updated).unwrap();
        assert_eq!(store.read_doc(&doc.id).unwrap().title, "new");
    }

    #[test]
    fn read_missing_doc_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path());
        match store.read_doc("missing") {
            Err(crate::Error::NotFound { id }) => assert_eq!(id, "missing"),
            other => panic!("unexpected {other:?}"),
        }
        assert!(!namespace_dir(dir.path()).exists());
    }
}
