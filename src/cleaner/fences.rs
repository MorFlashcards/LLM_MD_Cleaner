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

    let lang = fence_info(first, fence).to_ascii_lowercase();
    let is_markdownish = lang.is_empty() || matches!(lang.as_str(), "md" | "markdown" | "mdown");

    if !is_markdownish || !is_closing_fence(last, fence) {
        return input.to_string();
    }

    lines[first_index + 1..last_index].join("\n")
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

            if heading_re().is_match(trimmed) {
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

    // CommonMark forbids backticks in the info string of a backtick fence.
    // Treat those as ordinary text so we do not accidentally swallow content.
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

fn marker_run_len(input: &str, marker: char) -> usize {
    input
        .chars()
        .take_while(|candidate| *candidate == marker)
        .map(char::len_utf8)
        .sum()
}

fn heading_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| Regex::new(r"^\s{0,3}#{1,6}\s+\S").expect("valid Markdown heading regex"))
}
