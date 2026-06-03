use super::fences::{is_fence_line, is_inside_fence, update_fence_state, Fence};

pub(super) fn fix_lists(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut active_fence: Option<Fence> = None;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if update_fence_state(&mut active_fence, trimmed) {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        }

        if is_inside_fence(active_fence) {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        }

        if let Some(fixed) = fix_jammed_bullet_marker(lines[i]) {
            out.push(fixed);
            i += 1;
            continue;
        }

        if !is_loose_list_candidate(trimmed) {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        }

        let start = i;

        while i < lines.len() && is_loose_list_candidate(lines[i].trim()) {
            i += 1;
        }

        let block_len = i - start;

        if block_len >= 2 {
            for line in &lines[start..i] {
                out.push(format!("- {}", line.trim()));
            }
        } else {
            out.push(lines[start].to_string());
        }
    }

    out.join("\n")
}

fn fix_jammed_bullet_marker(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let leading = &line[..line.len() - trimmed.len()];
    let marker = trimmed.chars().next()?;

    if !matches!(marker, '-' | '*' | '+') {
        return None;
    }

    let rest = &trimmed[marker.len_utf8()..];

    if rest.is_empty()
        || rest.starts_with(char::is_whitespace)
        || rest.starts_with(marker)
        || (marker == '*' && rest.starts_with('/'))
    {
        return None;
    }

    let first = rest.chars().next()?;
    let starts_like_item = first.is_alphabetic() || first == '`' || first == '[';

    if starts_like_item {
        Some(format!("{leading}{marker} {rest}"))
    } else {
        None
    }
}

fn is_loose_list_candidate(trimmed: &str) -> bool {
    if trimmed.is_empty()
        || trimmed.len() > 100
        || is_fence_line(trimmed)
        || trimmed.starts_with('#')
        || trimmed.starts_with('>')
        || trimmed.starts_with('|')
        || trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || starts_with_numbered_list_marker(trimmed)
    {
        return false;
    }

    let starts_like_sentence_fragment = trimmed
        .chars()
        .next()
        .map(|ch| ch.is_ascii_lowercase())
        .unwrap_or(false);

    (starts_like_sentence_fragment && (trimmed.ends_with(',') || trimmed.ends_with(';')))
        || (trimmed.starts_with("no ") && (trimmed.ends_with(',') || trimmed.ends_with('.')))
}

fn starts_with_numbered_list_marker(trimmed: &str) -> bool {
    let mut chars = trimmed.chars().peekable();
    let mut saw_digit = false;

    while matches!(chars.peek(), Some(ch) if ch.is_ascii_digit()) {
        saw_digit = true;
        chars.next();
    }

    if !saw_digit {
        return false;
    }

    if !matches!(chars.next(), Some('.') | Some(')')) {
        return false;
    }

    matches!(chars.next(), Some(ch) if ch.is_whitespace())
}
