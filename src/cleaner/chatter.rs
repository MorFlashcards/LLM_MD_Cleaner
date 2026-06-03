use regex::Regex;
use std::sync::OnceLock;

pub(super) fn strip_llm_chatter(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut start = 0;
    let mut end = lines.len();

    while start < end {
        let trimmed = lines[start].trim();

        if trimmed.is_empty() || is_preamble(trimmed) {
            start += 1;
        } else {
            break;
        }
    }

    while end > start {
        let trimmed = lines[end - 1].trim();

        if trimmed.is_empty() || is_postamble(trimmed) {
            end -= 1;
        } else {
            break;
        }
    }

    lines[start..end].join("\n")
}

pub(super) fn remove_copy_code_artifacts(input: &str) -> String {
    input
        .lines()
        .filter(|line| !is_copy_code_artifact(line.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_preamble(trimmed: &str) -> bool {
    preamble_re().is_match(trimmed)
}

fn is_postamble(trimmed: &str) -> bool {
    postamble_re().is_match(trimmed)
}

fn is_copy_code_artifact(trimmed: &str) -> bool {
    // Keep this intentionally narrow. A line that only says "copy" can be
    // real content in documentation, commands, or copyright notes.
    matches!(
        trimmed.to_ascii_lowercase().as_str(),
        "copy code" | "copied" | "copied!"
    )
}

fn preamble_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| {
        Regex::new(
            r"(?ix)
            ^\s*
            (
                sure[,.!\s]+|
                certainly[,.!\s]+|
                of\s+course[,.!\s]+|
                here(?:'s|\s+is|\s+are)[\s:]+|
                below\s+is[\s:]+|
                i(?:'ve|\s+have)\s+
            )
            .*
            \b(markdown|cleaned|updated|version|code|file|draft|document|rewrite|revision)\b
            .*[:.!]?
            \s*$
            ",
        )
        .expect("valid LLM preamble regex")
    })
}

fn postamble_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| {
        Regex::new(
            r"(?ix)
            ^\s*
            (
                let\s+me\s+know\b.*|
                hope\s+this\s+helps\b.*|
                happy\s+to\s+help\b.*|
                thanks\s+for\s+using\s+chatgpt!?|
                that(?:'s|\s+is)\s+it[.!]?|
                done[.!]?
            )
            \s*$
            ",
        )
        .expect("valid LLM postamble regex")
    })
}
