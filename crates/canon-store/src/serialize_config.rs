pub fn to_yaml(locales: &[String]) -> String {
    if locales.is_empty() {
        return "{}\n".to_string();
    }
    let mut out = String::from("locales:\n");
    for locale in locales {
        out.push_str("  - ");
        out.push_str(locale);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::to_yaml;

    #[test]
    fn empty_is_flow_empty_object() {
        assert_eq!(to_yaml(&[]), "{}\n");
    }

    #[test]
    fn writes_locales_list() {
        assert_eq!(
            to_yaml(&["en".into(), "zh-Hans".into()]),
            "locales:\n  - en\n  - zh-Hans\n"
        );
    }
}
