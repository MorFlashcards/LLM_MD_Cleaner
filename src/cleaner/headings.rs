use super::fences::{is_inside_fence, update_fence_state, Fence};
use regex::Regex;
use std::sync::OnceLock;

/// Repair headings that were jammed into nearby prose by bad LLM/browser copies.
///
/// This pass is intentionally fence-aware. Code examples often contain `#`, so
/// heading cleanup must never run inside fenced blocks.
pub(super) fn fix_heading_jam(input: &str) -> String {
    let mut out = Vec::new();
    let mut active_fence: Option<Fence> = None;

    for line in input.lines() {
        let trimmed = line.trim();

        if update_fence_state(&mut active_fence, trimmed) {
            out.push(line.to_string());
            continue;
        }

        if is_inside_fence(active_fence) {
            out.push(line.to_string());
            continue;
        }

        for part in split_heading_marker_jam(line).lines() {
            out.extend(split_heading_body_jam(part));
        }
    }

    out.join("\n")
}

/// Split prose that runs directly into a Markdown heading marker:
///
/// `Paragraph text## Next Section` -> `Paragraph text\n\n## Next Section`
///
/// The character before `#` must not itself be `#`; otherwise clean headings
/// such as `## Status` get damaged into `# Status` after artifact cleanup.
fn split_heading_marker_jam(line: &str) -> String {
    heading_after_text_re()
        .replace_all(line, "$1\n\n$2")
        .to_string()
}

/// Split a heading whose title is jammed into its first paragraph:
///
/// `## FeaturesThe current version...` ->
/// `## Features\n\nThe current version...`
fn split_heading_body_jam(line: &str) -> Vec<String> {
    let Some(cap) = heading_re().captures(line) else {
        return vec![line.to_string()];
    };

    let prefix = cap.get(1).map(|m| m.as_str()).unwrap_or("");
    let body = cap.get(2).map(|m| m.as_str()).unwrap_or("");

    for (index, _) in body.char_indices().skip(4) {
        let rest = &body[index..];

        if starts_like_sentence_after_heading(body, index, rest) {
            let title = body[..index].trim_end();
            let paragraph = rest.trim_start();

            if !title.is_empty() && !paragraph.is_empty() {
                return vec![
                    format!("{prefix}{title}"),
                    String::new(),
                    paragraph.to_string(),
                ];
            }
        }
    }

    vec![line.to_string()]
}

fn starts_like_sentence_after_heading(body: &str, index: usize, rest: &str) -> bool {
    if rest.is_empty() {
        return false;
    }

    if starts_with_known_sentence_starter(rest) {
        return true;
    }

    // Conservative fallback for common LLM jams such as:
    //
    // # ProjectA tiny Rust app...
    // ## TitleOverview of the system...
    //
    // Split only at a lower->Upper+lower transition. That avoids breaking
    // acronym-heavy headings such as "OpenAI API".
    let before = body[..index].chars().last();
    let mut chars = rest.chars();
    let first = chars.next();
    let second = chars.next();

    matches!(before, Some(ch) if ch.is_lowercase())
        && matches!(first, Some(ch) if ch.is_uppercase())
        && matches!(second, Some(ch) if ch.is_lowercase())
}

fn starts_with_known_sentence_starter(rest: &str) -> bool {
    const SENTENCE_STARTERS: &[&str] = &[
        "The ",
        "This ",
        "These ",
        "Those ",
        "A ",
        "An ",
        "It ",
        "In ",
        "When ",
        "Where ",
        "Why ",
        "How ",
        "What ",
        "Note ",
        "Example ",
        "Overview ",
        "Use ",
        "For ",
    ];

    SENTENCE_STARTERS
        .iter()
        .any(|starter| rest.starts_with(starter))
}

fn heading_after_text_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| {
        Regex::new(r"([^#\s])(#{1,6}\s+)").expect("valid jammed heading marker regex")
    })
}

fn heading_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| {
        Regex::new(r"^(\s{0,3}#{1,6}\s+)(.+)$").expect("valid Markdown heading regex")
    })
}
