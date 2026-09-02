const FORBIDDEN: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Agent-supplied slug used as the atom id. Not a separate persisted field.
pub fn validate_slug(slug: &str) -> Result<(), String> {
    if slug.is_empty() {
        return Err("slug must be non-empty".into());
    }
    let chars: Vec<char> = slug.chars().collect();
    if chars.len() > 32 {
        return Err("slug must be at most 32 Unicode scalars".into());
    }
    let first = chars[0];
    let last = chars[chars.len() - 1];
    if first.is_whitespace()
        || last.is_whitespace()
        || first == '.'
        || last == '.'
        || first == '_'
        || last == '_'
    {
        return Err("slug must not start or end with whitespace, `.`, or `_`".into());
    }
    if chars.iter().any(|c| FORBIDDEN.contains(c)) {
        return Err("slug contains a forbidden character".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_slug;

    #[test]
    fn accepts_snake_case_english_slug() {
        assert!(validate_slug("durability_daily_restore").is_ok());
        assert!(validate_slug("a_b").is_ok());
    }

    #[test]
    fn still_accepts_unicode_without_underscore() {
        assert!(validate_slug("禁军突围耐久恢复").is_ok());
    }

    #[test]
    fn accepts_32_scalars() {
        let slug: String = "a".repeat(32);
        assert_eq!(slug.chars().count(), 32);
        assert!(validate_slug(&slug).is_ok());
    }

    #[test]
    fn rejects_empty_edges_too_long_and_path_chars() {
        assert!(validate_slug("").is_err());
        assert!(validate_slug(&"a".repeat(33)).is_err());
        assert!(validate_slug("_leading").is_err());
        assert!(validate_slug("trailing_").is_err());
        assert!(validate_slug(".leading").is_err());
        assert!(validate_slug("trailing.").is_err());
        assert!(validate_slug(" spaced").is_err());
        assert!(validate_slug("spaced ").is_err());
        assert!(validate_slug("a/b").is_err());
        assert!(validate_slug("a:b").is_err());
        assert!(validate_slug("a*b").is_err());
    }
}
