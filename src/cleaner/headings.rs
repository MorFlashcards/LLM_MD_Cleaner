use super::fences::{is_inside_fence, update_fence_state, Fence};
use regex::Regex;
use std::sync::OnceLock;

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

        let split_line = heading_after_text_re()
            .replace_all(line, "$1\n\n$2")
            .to_string();

        for part in split_line.lines() {
            out.extend(split_jammed_heading_line(part));
        }
    }

    out.join("\n")
}

fn split_jammed_heading_line(line: &str) -> Vec<String> {
    let Some(cap) = heading_re().captures(line) else {
        return vec![line.to_string()];
    };

    let prefix = cap.get(1).map(|m| m.as_str()).unwrap_or("");
    let body = cap.get(2).map(|m| m.as_str()).unwrap_or("");

    for (index, _) in body.char_indices().skip(4) {
        let rest = &body[index..];

        if starts_like_sentence_after_heading(rest) {
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

fn starts_like_sentence_after_heading(rest: &str) -> bool {
    const SENTENCE_STARTERS: &[&str] = &[
        "The ", "This ", "These ", "Those ", "A ", "An ", "It ", "In ", "When ", "Where ", "Why ",
        "How ", "What ", "Note ", "Example ",
    ];

    SENTENCE_STARTERS
        .iter()
        .any(|starter| rest.starts_with(starter))
}

fn heading_after_text_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| Regex::new(r"(\S)(#{1,6}\s+)").expect("valid jammed heading splitter regex"))
}

fn heading_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| {
        Regex::new(r"^(\s{0,3}#{1,6}\s+)(.+)$").expect("valid Markdown heading regex")
    })
}
