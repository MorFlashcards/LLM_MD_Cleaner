use regex::Regex;

pub fn clean_markdown(input: &str) -> String {
    let mut text = input.to_string();

    text = prefer_markdown_tail(&text);
    text = html_escape::decode_html_entities(&text).to_string();

    text = Regex::new(r"(?i)<br\s*/?>")
        .unwrap()
        .replace_all(&text, "\n")
        .to_string();

    text = extract_code_block_text(&text);
    text = strip_html_tags(&text);
    text = strip_llm_chatter(&text);
    text = remove_copy_code_artifacts(&text);
    text = unwrap_outer_markdown_fence(&text);
    text = fix_heading_jam(&text);
    text = fix_unclosed_code_fences(&text);
    text = convert_tsv_tables(&text);
    text = fix_loose_lists(&text);

    text = Regex::new(r"[ \t]+\n")
        .unwrap()
        .replace_all(&text, "\n")
        .to_string();

    text = Regex::new(r"\n{3,}")
        .unwrap()
        .replace_all(&text, "\n\n")
        .to_string();

    text.trim().to_string()
}

fn prefer_markdown_tail(input: &str) -> String {
    for marker in ["</pre>", "</div>"] {
        if let Some(index) = input.rfind(marker) {
            let tail = input[index + marker.len()..].trim();
            if tail.starts_with("# ") {
                return tail.to_string();
            }
        }
    }

    input.to_string()
}

fn extract_code_block_text(input: &str) -> String {
    let span_re = Regex::new(r"(?is)<span[^>]*>(.*?)</span>").unwrap();

    if input.contains("cm-content") && input.contains("<span") {
        let mut out = String::new();

        for cap in span_re.captures_iter(input) {
            let line = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = html_escape::decode_html_entities(line);
            out.push_str(&line);
            out.push('\n');
        }

        if !out.trim().is_empty() {
            return out;
        }
    }

    input.to_string()
}

fn strip_html_tags(input: &str) -> String {
    let mut text = input.to_string();

    text = Regex::new(r"(?is)<script.*?</script>")
        .unwrap()
        .replace_all(&text, "")
        .to_string();

    text = Regex::new(r"(?is)<style.*?</style>")
        .unwrap()
        .replace_all(&text, "")
        .to_string();

    text = Regex::new(
        r"(?is)</?(h1|h2|h3|h4|h5|h6|p|div|li|ul|ol|pre|code|span|table|tbody|thead|tr|td|th|a)[^>]*>",
    )
    .unwrap()
    .replace_all(&text, "\n")
    .to_string();

    text = Regex::new(r"(?is)<[^>]+>")
        .unwrap()
        .replace_all(&text, "")
        .to_string();

    text
}

fn strip_llm_chatter(input: &str) -> String {
    let preamble_re = Regex::new(
        r"(?ix)
        ^\s*(
            sure[,.!\s]+|
            certainly[,.!\s]+|
            of\s+course[,.!\s]+|
            here(?:'s|\s+is|\s+are)[\s:]+|
            below\s+is[\s:]+|
            i(?:'ve|\s+have)\s+
        ).*\b(markdown|cleaned|updated|version|code|file|draft)\b.*[:.!]?\s*$
        ",
    )
    .unwrap();

    let postamble_re = Regex::new(
        r"(?ix)
        ^\s*(
            let\s+me\s+know\b.*|
            hope\s+this\s+helps\b.*|
            happy\s+to\s+help\b.*|
            that(?:'s|\s+is)\s+it[.!]?|
            done[.!]?
        )\s*$
        ",
    )
    .unwrap();

    let lines: Vec<&str> = input.lines().collect();
    let mut start = 0;
    let mut end = lines.len();

    while start < end {
        let trimmed = lines[start].trim();

        if trimmed.is_empty() || preamble_re.is_match(trimmed) {
            start += 1;
        } else {
            break;
        }
    }

    while end > start {
        let trimmed = lines[end - 1].trim();

        if trimmed.is_empty() || postamble_re.is_match(trimmed) {
            end -= 1;
        } else {
            break;
        }
    }

    lines[start..end].join("\n")
}

fn remove_copy_code_artifacts(input: &str) -> String {
    input
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !matches!(
                trimmed,
                "Copy code" | "Copy Code" | "COPY CODE" | "Copied!" | "Copied" | "copied"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn unwrap_outer_markdown_fence(input: &str) -> String {
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
    let Some(marker) = fence_marker(first) else {
        return input.to_string();
    };

    let lang = first[marker.len()..].trim().to_ascii_lowercase();
    let is_markdownish =
        lang.is_empty() || matches!(lang.as_str(), "md" | "markdown" | "mdown" | "text" | "txt");

    if !is_markdownish || !last.starts_with(marker) || !last[marker.len()..].trim().is_empty() {
        return input.to_string();
    }

    lines[first_index + 1..last_index].join("\n")
}

fn fix_heading_jam(input: &str) -> String {
    let heading_after_text_re = Regex::new(r"(\S)(#{1,6}\s+)").unwrap();
    let mut out = Vec::new();
    let mut active_fence: Option<&'static str> = None;

    for line in input.lines() {
        let trimmed = line.trim();

        if let Some(marker) = fence_marker(trimmed) {
            match active_fence {
                Some(active) if marker == active => active_fence = None,
                None => active_fence = Some(marker),
                _ => {}
            }

            out.push(line.to_string());
            continue;
        }

        if active_fence.is_some() {
            out.push(line.to_string());
            continue;
        }

        let split_line = heading_after_text_re
            .replace_all(line, "$1\n\n$2")
            .to_string();

        for part in split_line.lines() {
            out.extend(split_jammed_heading_line(part));
        }
    }

    out.join("\n")
}

fn split_jammed_heading_line(line: &str) -> Vec<String> {
    let heading_re = Regex::new(r"^(\s{0,3}#{1,6}\s+)(.+)$").unwrap();
    let Some(cap) = heading_re.captures(line) else {
        return vec![line.to_string()];
    };

    let prefix = cap.get(1).map(|m| m.as_str()).unwrap_or("");
    let body = cap.get(2).map(|m| m.as_str()).unwrap_or("");

    let sentence_starters = [
        "The ", "This ", "These ", "Those ", "A ", "An ", "It ", "In ", "When ", "Where ", "Why ",
        "How ", "What ", "Note ", "Example ",
    ];

    for (index, _) in body.char_indices().skip(4) {
        let rest = &body[index..];

        if sentence_starters
            .iter()
            .any(|starter| rest.starts_with(starter))
        {
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

fn fix_unclosed_code_fences(input: &str) -> String {
    let heading_re = Regex::new(r"^\s{0,3}#{1,6}\s+\S").unwrap();
    let mut out = Vec::new();
    let mut active_fence: Option<String> = None;

    for line in input.lines() {
        let trimmed = line.trim();

        if let Some(marker) = fence_marker(trimmed) {
            match &active_fence {
                Some(active) if marker == active => active_fence = None,
                None => active_fence = Some(marker.to_string()),
                _ => {}
            }

            out.push(line.to_string());
            continue;
        }

        if let Some(active) = &active_fence {
            if heading_re.is_match(trimmed) {
                out.push(active.clone());
                out.push(String::new());
                active_fence = None;
            }
        }

        out.push(line.to_string());
    }

    if let Some(active) = active_fence {
        out.push(active);
    }

    out.join("\n")
}

fn fence_marker(trimmed: &str) -> Option<&'static str> {
    if trimmed.starts_with("```") {
        Some("```")
    } else if trimmed.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

fn convert_tsv_tables(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut active_fence: Option<&'static str> = None;
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if let Some(marker) = fence_marker(trimmed) {
            match active_fence {
                Some(active) if marker == active => active_fence = None,
                None => active_fence = Some(marker),
                _ => {}
            }

            out.push(lines[i].to_string());
            i += 1;
            continue;
        }

        if active_fence.is_some() || !lines[i].contains('\t') {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        }

        let mut block = Vec::new();

        while i < lines.len() && lines[i].contains('\t') {
            block.push(lines[i]);
            i += 1;
        }

        if block.len() >= 2 {
            out.extend(tsv_block_to_markdown_table(&block));
        } else {
            out.extend(block.into_iter().map(str::to_string));
        }
    }

    out.join("\n")
}

fn tsv_block_to_markdown_table(block: &[&str]) -> Vec<String> {
    let rows: Vec<Vec<String>> = block
        .iter()
        .map(|row| row.split('\t').map(clean_table_cell).collect())
        .collect();

    let max_cols = rows.iter().map(Vec::len).max().unwrap_or(0);

    if max_cols < 2 {
        return block.iter().map(|row| row.to_string()).collect();
    }

    let mut out = Vec::new();
    out.push(format_table_row(&rows[0], max_cols));
    out.push(format_table_separator(max_cols));

    for row in rows.iter().skip(1) {
        out.push(format_table_row(row, max_cols));
    }

    out
}

fn clean_table_cell(cell: &str) -> String {
    cell.trim().replace('|', r"\|")
}

fn format_table_row(row: &[String], max_cols: usize) -> String {
    let cells = (0..max_cols)
        .map(|index| row.get(index).map(String::as_str).unwrap_or(""))
        .collect::<Vec<_>>()
        .join(" | ");

    format!("| {cells} |")
}

fn format_table_separator(max_cols: usize) -> String {
    format!("|{}|", vec!["---"; max_cols].join("|"))
}

fn fix_loose_lists(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut active_fence: Option<&'static str> = None;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if let Some(marker) = fence_marker(trimmed) {
            match active_fence {
                Some(active) if marker == active => active_fence = None,
                None => active_fence = Some(marker),
                _ => {}
            }

            out.push(lines[i].to_string());
            i += 1;
            continue;
        }

        if active_fence.is_some() || !is_loose_list_candidate(trimmed) {
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

fn is_fence_line(trimmed: &str) -> bool {
    fence_marker(trimmed).is_some()
}

#[cfg(test)]
mod tests {
    use super::clean_markdown;

    #[test]
    fn leaves_inline_code_alone() {
        let input = "Use `word` inline.";
        assert_eq!(clean_markdown(input), "Use `word` inline.");
    }

    #[test]
    fn closes_odd_code_fence_at_end() {
        let input = "```bash\necho hello";
        assert_eq!(clean_markdown(input), "```bash\necho hello\n```");
    }

    #[test]
    fn closes_fence_before_markdown_heading() {
        let input = "```bash\necho hello\n## Next Section\nText.";
        assert_eq!(
            clean_markdown(input),
            "```bash\necho hello\n```\n\n## Next Section\nText."
        );
    }

    #[test]
    fn strips_basic_llm_chatter() {
        let input = "Sure, here is the cleaned markdown:\n\n# Title\n\nBody\n\nLet me know if you need anything else.";
        assert_eq!(clean_markdown(input), "# Title\n\nBody");
    }

    #[test]
    fn removes_copy_code_artifacts() {
        let input = "Copy code\n```rust\nfn main() {}\n```\nCopied!";
        assert_eq!(clean_markdown(input), "```rust\nfn main() {}\n```");
    }

    #[test]
    fn converts_multi_column_tsv_tables() {
        let input = "Name\tKind\tNote\nRust\tLanguage\tFast";
        assert_eq!(
            clean_markdown(input),
            "| Name | Kind | Note |\n|---|---|---|\n| Rust | Language | Fast |"
        );
    }
}
