use regex::Regex;
use std::sync::OnceLock;

/// Remove copied-output artifacts that are valid-looking Markdown but useless.
///
/// This removes only empty heading lines like "#", "##", or "###".
/// It must never rewrite real headings like "## Section".
pub(super) fn remove_artifacts(input: &str) -> String {
    input
        .lines()
        .filter(|line| !is_bare_empty_heading(line.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_bare_empty_heading(trimmed: &str) -> bool {
    bare_empty_heading_re().is_match(trimmed)
}

fn bare_empty_heading_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| Regex::new(r"^#{1,6}\s*$").expect("valid bare empty heading artifact regex"))
}
