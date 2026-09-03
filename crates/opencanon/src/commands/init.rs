use std::fs;
use std::io;
use std::path::Path;

use canon_store::Store;
use include_dir::{include_dir, Dir};
use serde_json::{json, Value};

use crate::error::CliError;

static BUNDLED_SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../skills");

struct LocaleChoice {
    code: &'static str,
    label: &'static str,
}

const LOCALE_CHOICES: &[LocaleChoice] = &[
    LocaleChoice {
        code: "en",
        label: "English",
    },
    LocaleChoice {
        code: "zh-Hans",
        label: "简体中文",
    },
    LocaleChoice {
        code: "zh-Hant",
        label: "繁體中文",
    },
    LocaleChoice {
        code: "ja",
        label: "日本語",
    },
    LocaleChoice {
        code: "ko",
        label: "한국어",
    },
    LocaleChoice {
        code: "de",
        label: "Deutsch",
    },
    LocaleChoice {
        code: "fr",
        label: "Français",
    },
    LocaleChoice {
        code: "es",
        label: "Español",
    },
    LocaleChoice {
        code: "pt",
        label: "Português",
    },
    LocaleChoice {
        code: "ru",
        label: "Русский",
    },
    LocaleChoice {
        code: "ar",
        label: "العربية",
    },
    LocaleChoice {
        code: "hi",
        label: "हिन्दी",
    },
];

pub fn run(store: &Store) -> Result<Value, CliError> {
    let labels: Vec<String> = LOCALE_CHOICES
        .iter()
        .map(|choice| format!("{} ({})", choice.label, choice.code))
        .collect();
    let selected = dialoguer::MultiSelect::new()
        .with_prompt("Select document languages (space to toggle, enter to confirm)")
        .items(&labels)
        .interact()
        .map_err(|e| CliError::Io {
            message: e.to_string(),
        })?;
    let locales = locales_from_indices(&selected);
    apply(store, &locales)
}

pub fn apply(store: &Store, locales: &[String]) -> Result<Value, CliError> {
    let mut locales = locales.to_vec();
    locales.sort();
    locales.dedup();
    store
        .init_namespace()
        .map_err(|e| CliError::from_store(e, None))?;
    store
        .write_config(&locales)
        .map_err(|e| CliError::from_store(e, None))?;
    let skills = install_bundled_skills(store.root()).map_err(|e| CliError::Io {
        message: e.to_string(),
    })?;
    Ok(json!({
        "locales": locales,
        "skills": skills,
    }))
}

fn locales_from_indices(indices: &[usize]) -> Vec<String> {
    let mut locales: Vec<String> = indices
        .iter()
        .filter_map(|index| {
            LOCALE_CHOICES
                .get(*index)
                .map(|choice| choice.code.to_string())
        })
        .collect();
    locales.sort();
    locales.dedup();
    locales
}

fn install_bundled_skills(root: &Path) -> io::Result<Vec<String>> {
    install_skills_from(root, &BUNDLED_SKILLS)
}

fn install_skills_from(root: &Path, bundled: &Dir) -> io::Result<Vec<String>> {
    let dest_parent = root.join(".agents").join("skills");
    fs::create_dir_all(&dest_parent)?;
    let mut names = Vec::new();
    for dir in bundled.dirs() {
        if !dir
            .files()
            .any(|file| file.path().file_name().and_then(|n| n.to_str()) == Some("SKILL.md"))
        {
            continue;
        }
        let Some(name) = dir.path().file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let dest = dest_parent.join(name);
        if dest.exists() {
            fs::remove_dir_all(&dest)?;
        }
        write_skill_dir(dir, &dest)?;
        names.push(name.to_string());
    }
    names.sort();
    Ok(names)
}

fn write_skill_dir(dir: &Dir, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    for file in dir.files() {
        let Some(name) = file.path().file_name() else {
            continue;
        };
        fs::write(dest.join(name), file.contents())?;
    }
    for child in dir.dirs() {
        let Some(name) = child.path().file_name() else {
            continue;
        };
        write_skill_dir(child, &dest.join(name))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use canon_store::Store;

    use super::{apply, install_skills_from, locales_from_indices, BUNDLED_SKILLS, LOCALE_CHOICES};

    #[test]
    fn selection_sorts_and_dedups_codes() {
        assert_eq!(locales_from_indices(&[0]), vec!["en".to_string()]);
        assert_eq!(
            locales_from_indices(&[1, 0, 1]),
            vec!["en".to_string(), "zh-Hans".to_string()]
        );
        assert!(locales_from_indices(&[]).is_empty());
        assert_eq!(LOCALE_CHOICES[0].code, "en");
    }

    #[test]
    fn install_overwrites_same_name_and_keeps_other_skills() {
        let dir = tempfile::tempdir().unwrap();
        let skills = dir.path().join(".agents").join("skills");
        let other = skills.join("other-skill");
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(other.join("SKILL.md"), "keep me").unwrap();
        let stale = skills.join("opencanon-atomize");
        std::fs::create_dir_all(stale.join("stale-dir")).unwrap();
        std::fs::write(stale.join("stale.md"), "old").unwrap();

        let names = install_skills_from(dir.path(), &BUNDLED_SKILLS).unwrap();
        assert!(names.contains(&"opencanon-atomize".to_string()));
        assert!(names.contains(&"opencanon-compose".to_string()));
        assert!(!names.contains(&"other-skill".to_string()));
        assert_eq!(
            std::fs::read_to_string(other.join("SKILL.md")).unwrap(),
            "keep me"
        );
        assert!(skills.join("opencanon-atomize").join("SKILL.md").exists());
        assert!(skills
            .join("opencanon-atomize")
            .join("references")
            .join("query.md")
            .exists());
        assert!(!stale.join("stale.md").exists());
    }

    #[test]
    fn apply_sorts_locales_and_creates_namespace() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path());
        let data = apply(&store, &["zh-Hans".into(), "en".into()]).unwrap();
        assert_eq!(data["locales"], serde_json::json!(["en", "zh-Hans"]));
        let config =
            std::fs::read_to_string(dir.path().join("opencanon").join("config.yaml")).unwrap();
        assert_eq!(config, "locales:\n  - en\n  - zh-Hans\n");
        assert!(dir.path().join("opencanon").join("atoms").is_dir());
        assert!(!dir.path().join("opencanon").join("docs").exists());
    }
}
