use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Fence {
    marker: char,
    len: usize,
}

impl Fence {
    fn closing_text(self) -> String {
        self.marker.to_string().repeat(self.len)
    }
}

pub(super) fn unwrap_outer_markdown_fence(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();

    let Some(first_index) = lines.iter().position(|line| !line.trim().is_empty()) else {
        return input.to_string();
    };
    let Some(last_index) = lines.iter().rposition(|line| !line.trim().is_empty()) else {
        return input.to_string();
    };

    if first_index >= last_index {
        return input.to_string();
    }

    let first = lines[first_index].trim();
    let last = lines[last_index].trim();
    let Some(fence) = parse_opening_fence(first) else {
        return input.to_string();
    };

    let info = fence_info(first, fence).to_ascii_lowercase();

    if !is_markdownish_fence_info(&info) || !is_closing_fence(last, fence) {
        return input.to_string();
    }

    lines[first_index + 1..last_index].join("\n")
}

/// Remove empty Markdown wrapper fences left behind by copied LLM output.
///
/// Only Markdown-ish empty fences are removed. Real empty examples like
/// `rust`, `bash`, `text`, `xml`, or `toml` fences are preserved.
pub(super) fn remove_empty_markdown_fences(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        let Some(fence) = parse_opening_fence(trimmed) else {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        };

        let info = fence_info(trimmed, fence).to_ascii_lowercase();

        if !is_markdownish_fence_info(&info) {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        }

        let mut closing_index = i + 1;

        while closing_index < lines.len() && lines[closing_index].trim().is_empty() {
            closing_index += 1;
        }

        if closing_index < lines.len() && is_closing_fence(lines[closing_index].trim(), fence) {
            i = closing_index + 1;
        } else {
            out.push(lines[i].to_string());
            i += 1;
        }
    }

    out.join("\n")
}

pub(super) fn fix_unclosed_code_fences(input: &str) -> String {
    let mut out = Vec::new();
    let mut active_fence: Option<Fence> = None;

    for line in input.lines() {
        let trimmed = line.trim();

        if let Some(active) = active_fence {
            if is_closing_fence(trimmed, active) {
                active_fence = None;
                out.push(line.to_string());
                continue;
            }

            if is_fence_escape_heading(trimmed) {
                out.push(active.closing_text());
                out.push(String::new());
                active_fence = None;
            }

            out.push(line.to_string());
            continue;
        }

        if let Some(fence) = parse_opening_fence(trimmed) {
            active_fence = Some(fence);
            out.push(line.to_string());
            continue;
        }

        out.push(line.to_string());
    }

    if let Some(active) = active_fence {
        out.push(active.closing_text());
    }

    out.join("\n")
}

pub(super) fn update_fence_state(active_fence: &mut Option<Fence>, trimmed: &str) -> bool {
    if let Some(active) = *active_fence {
        if is_closing_fence(trimmed, active) {
            *active_fence = None;
            return true;
        }

        return false;
    }

    if let Some(fence) = parse_opening_fence(trimmed) {
        *active_fence = Some(fence);
        return true;
    }

    false
}

pub(super) fn is_inside_fence(active_fence: Option<Fence>) -> bool {
    active_fence.is_some()
}

pub(super) fn is_fence_line(trimmed: &str) -> bool {
    parse_opening_fence(trimmed).is_some()
}

fn parse_opening_fence(trimmed: &str) -> Option<Fence> {
    let marker = trimmed.chars().next()?;

    if marker != '`' && marker != '~' {
        return None;
    }

    let len = marker_run_len(trimmed, marker);

    if len < 3 {
        return None;
    }

    let info = trimmed[len..].trim();

    // CommonMark forbids backticks in a backtick fence info string.
    // Treat those lines as normal text so inline/backtick-heavy text is not swallowed.
    if marker == '`' && info.contains('`') {
        return None;
    }

    Some(Fence { marker, len })
}

fn is_closing_fence(trimmed: &str, fence: Fence) -> bool {
    if !trimmed.starts_with(fence.marker) {
        return false;
    }

    let len = marker_run_len(trimmed, fence.marker);
    len >= fence.len && trimmed[len..].trim().is_empty()
}

fn fence_info(trimmed: &str, fence: Fence) -> &str {
    trimmed[fence.len..].trim()
}

fn is_markdownish_fence_info(info: &str) -> bool {
    info.is_empty() || matches!(info, "md" | "markdown" | "mdown")
}

fn is_fence_escape_heading(trimmed: &str) -> bool {
    fence_escape_heading_re().is_match(trimmed)
}

fn marker_run_len(input: &str, marker: char) -> usize {
    input
        .chars()
        .take_while(|candidate| *candidate == marker)
        .map(char::len_utf8)
        .sum()
}

fn fence_escape_heading_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    // H1 is too common inside shell/config comments. H2-H6 are a stronger
    // signal that an LLM forgot to close a code fence before the next section.
    RE.get_or_init(|| Regex::new(r"^\s{0,3}#{2,6}\s+\S").expect("valid fence escape heading regex"))
}
