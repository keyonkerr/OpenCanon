use crate::model::Atom;

use super::super::ImplSnapshot;

pub const ID: &str = "body-in-impl";

pub fn value(atom: &Atom, snapshot: &ImplSnapshot) -> f64 {
    if !snapshot.exists {
        return 0.0;
    }
    let Some(text) = snapshot.text.as_deref() else {
        return 0.0;
    };
    let tokens = code_tokens(&atom.body);
    if tokens.is_empty() {
        return 1.0;
    }
    let haystack = text.to_ascii_lowercase();
    if tokens.iter().all(|tok| haystack.contains(tok)) {
        1.0
    } else {
        0.0
    }
}

/// `[A-Za-z_][A-Za-z0-9_]*` with length ≥ 3, unique, case-folded.
fn code_tokens(body: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = body.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            let mut end = i + c.len_utf8();
            while let Some((_, n)) = chars.peek() {
                if n.is_ascii_alphanumeric() || *n == '_' {
                    let (j, n) = chars.next().unwrap();
                    end = j + n.len_utf8();
                } else {
                    break;
                }
            }
            let tok = &body[start..end];
            if tok.len() >= 3 {
                let folded = tok.to_ascii_lowercase();
                if !tokens.contains(&folded) {
                    tokens.push(folded);
                }
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::code_tokens;

    #[test]
    fn extracts_code_like_words() {
        assert_eq!(
            code_tokens("call FooBar and max_hp plus get."),
            vec!["call", "foobar", "and", "max_hp", "plus", "get"]
        );
        assert_eq!(
            code_tokens("纯中文主张，没有代码名。"),
            Vec::<String>::new()
        );
    }
}
