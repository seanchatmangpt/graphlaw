//! Security guard: comment-aware SQL validation and ATTACH root policy enforcement.
//! Resolves E1 vulnerability and enforces REFUSAL-06 (SQLCommentBypassAttempt).

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityRefusal {
    AttachOutsideRoots(String),
    CommentBypassAttempt(String),
    MalformedSql(String),
}

impl std::fmt::Display for SecurityRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AttachOutsideRoots(path) => {
                write!(f, "attach path outside GRAPHLAW_ATTACH_ROOTS allowlist: {path}")
            }
            Self::CommentBypassAttempt(reason) => {
                write!(f, "REFUSAL-06: SQL comment bypass attempt detected: {reason}")
            }
            Self::MalformedSql(reason) => {
                write!(f, "malformed SQL: {reason}")
            }
        }
    }
}

impl std::error::Error for SecurityRefusal {}

/// Strip both line comments (`-- ...\n`) and block comments (`/* ... */`, including multiline)
/// from SQL text, returning the purely semantic tokens.
pub fn strip_sql_comments(sql: &str) -> Result<(String, bool), SecurityRefusal> {
    let mut chars = sql.chars().peekable();
    let mut out = String::with_capacity(sql.len());
    let mut had_comments = false;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(c) = chars.next() {
        if in_single_quote {
            out.push(c);
            if c == '\'' {
                if chars.peek() == Some(&'\'') {
                    out.push(chars.next().unwrap()); // escaped quote
                } else {
                    in_single_quote = false;
                }
            }
            continue;
        }

        if in_double_quote {
            out.push(c);
            if c == '"' {
                in_double_quote = false;
            }
            continue;
        }

        match c {
            '\'' => {
                in_single_quote = true;
                out.push(c);
            }
            '"' => {
                in_double_quote = true;
                out.push(c);
            }
            '-' if chars.peek() == Some(&'-') => {
                had_comments = true;
                chars.next(); // consume second '-'
                while let Some(n) = chars.next() {
                    if n == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                had_comments = true;
                chars.next(); // consume '*'
                let mut depth = 1usize;
                while depth > 0 {
                    match chars.next() {
                        Some('*') if chars.peek() == Some(&'/') => {
                            chars.next();
                            depth -= 1;
                        }
                        Some('/') if chars.peek() == Some(&'*') => {
                            chars.next();
                            depth += 1;
                        }
                        Some(_) => {}
                        None => {
                            return Err(SecurityRefusal::MalformedSql(
                                "unclosed block comment in SQL text".into(),
                            ));
                        }
                    }
                }
            }
            _ => out.push(c),
        }
    }

    Ok((out, had_comments))
}

/// Enforce ATTACH path allowlist against allowed root paths.
pub fn validate_sql_security(sql: &str, allowed_roots: Option<&str>) -> Result<(), SecurityRefusal> {
    let (stripped, had_comments) = strip_sql_comments(sql)?;
    let trimmed = stripped.trim_start();

    // Extract first keyword token
    let first_token: String = trimmed
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_uppercase();

    if first_token == "ATTACH" {
        // If SQL comments were explicitly placed ahead of ATTACH to obfuscate the command,
        // flag REFUSAL-06
        let raw_trimmed = sql.trim_start();
        if had_comments && !raw_trimmed.starts_with("ATTACH") && !raw_trimmed.starts_with("attach") {
            return Err(SecurityRefusal::CommentBypassAttempt(
                "ATTACH keyword concealed behind comments".into(),
            ));
        }

        let path = extract_attach_path(&stripped)?;
        check_path_allowlist(&path, allowed_roots)?;
    }

    Ok(())
}

fn extract_attach_path(sql: &str) -> Result<String, SecurityRefusal> {
    let mut chars = sql.chars().peekable();
    let mut in_quote = false;
    let mut current = String::new();

    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                if in_quote && chars.peek() == Some(&'\'') {
                    current.push('\'');
                    chars.next();
                } else if in_quote {
                    return Ok(current);
                } else {
                    in_quote = true;
                    current.clear();
                }
            }
            _ if in_quote => current.push(c),
            _ => {}
        }
    }
    Err(SecurityRefusal::AttachOutsideRoots("<none>".into()))
}

fn check_path_allowlist(path: &str, allowed_roots: Option<&str>) -> Result<(), SecurityRefusal> {
    let Some(roots) = allowed_roots else {
        return Err(SecurityRefusal::AttachOutsideRoots(path.to_string()));
    };
    if roots.trim().is_empty() {
        return Err(SecurityRefusal::AttachOutsideRoots(path.to_string()));
    }

    let allowed: Vec<&str> = roots.split(':').filter(|s| !s.is_empty()).collect();
    for root in allowed {
        if path.starts_with(root) {
            return Ok(());
        }
    }
    Err(SecurityRefusal::AttachOutsideRoots(path.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_stripping() {
        let (stripped, had) = strip_sql_comments("/* comment */ SELECT 1;").unwrap();
        assert!(had);
        assert_eq!(stripped.trim(), "SELECT 1;");
    }

    #[test]
    fn test_attach_bypass_flagged_refusal_06() {
        let sql = "/* Harmless */ ATTACH '/etc/passwd' AS stolen;";
        let err = validate_sql_security(sql, Some("/var/lib/db")).unwrap_err();
        assert!(matches!(err, SecurityRefusal::CommentBypassAttempt(_)));
    }

    #[test]
    fn test_valid_attach_allowed() {
        let sql = "ATTACH '/var/lib/db/core.db' AS core;";
        assert!(validate_sql_security(sql, Some("/var/lib/db")).is_ok());
    }

    #[test]
    fn test_attach_outside_roots_refused() {
        let sql = "ATTACH '/tmp/outside.db' AS ext;";
        let err = validate_sql_security(sql, Some("/var/lib/db")).unwrap_err();
        assert!(matches!(err, SecurityRefusal::AttachOutsideRoots(_)));
    }
}
