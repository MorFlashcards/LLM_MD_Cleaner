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
    text = fix_heading_jam(&text);
    text = fix_known_headings(&text);
    text = fix_unclosed_code_fences(&text);
    text = convert_tsv_tables(&text);
    text = fix_loose_lists(&text);
    text = improve_common_wiki_links(&text);

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

fn fix_heading_jam(input: &str) -> String {
    input.replace(
        "# Root DirectoryThe root directory",
        "# Root Directory\n\nThe root directory",
    )
}

fn fix_known_headings(input: &str) -> String {
    let known = [
        "Current Root Layout",
        "Why the Root Directory Matters",
        "Design Principle",
        "Visual Map",
        "Screenshot Storage",
        "Root Directory Philosophy",
        "Related Pages",
        "Why Screenshots Belong in the Wiki",
    ];

    let file_headings = [
        "docs/",
        "src/",
        "ui/",
        "build.rs",
        "Cargo.toml",
        ".gitignore",
        ".git/",
    ];

    let mut out = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            out.push(line.to_string());
        } else if known.contains(&trimmed) {
            out.push(format!("## {trimmed}"));
        } else if file_headings.contains(&trimmed) {
            out.push(format!("## `{trimmed}`"));
        } else {
            out.push(line.to_string());
        }
    }

    out.join("\n")
}

fn fix_unclosed_code_fences(input: &str) -> String {
    let known_headings = [
        "## Why the Root Directory Matters",
        "## Design Principle",
        "## Visual Map",
        "## Screenshot Storage",
        "## Root Directory Philosophy",
        "## Related Pages",
        "## `docs/`",
        "## `src/`",
        "## `ui/`",
        "## `build.rs`",
        "## `Cargo.toml`",
        "## `.gitignore`",
        "## `.git/`",
    ];

    let mut out = Vec::new();
    let mut in_fence = false;

    for line in input.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            out.push(line.to_string());
            continue;
        }

        if in_fence && known_headings.contains(&trimmed) {
            out.push("```".to_string());
            out.push(String::new());
            in_fence = false;
        }

        out.push(line.to_string());
    }

    if in_fence {
        out.push("```".to_string());
    }

    out.join("\n")
}

fn convert_tsv_tables(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if !lines[i].contains('\t') {
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
            let header: Vec<&str> = block[0].split('\t').collect();

            if header.len() >= 2 {
                out.push(format!("| {} | {} |", header[0].trim(), header[1].trim()));
                out.push("|---|---|".to_string());

                for row in block.iter().skip(1) {
                    let cells: Vec<&str> = row.split('\t').collect();

                    if cells.len() >= 2 {
                        let first = cells[0].trim();
                        let second = cells[1..].join(" ");
                        out.push(format!("| {first} | {} |", second.trim()));
                    }
                }
            } else {
                for row in block {
                    out.push(row.to_string());
                }
            }
        } else {
            for row in block {
                out.push(row.to_string());
            }
        }
    }

    out.join("\n")
}

fn fix_loose_lists(input: &str) -> String {
    let bullet_items = [
        "where files live,",
        "how folders relate to each other,",
        "what a beginner should open first,",
        "how the project evolves over time,",
        "why a file exists before the reader sees any Rust code.",
        "no browser shell,",
        "no giant frontend framework,",
        "no mystery build folder,",
        "no schema graveyard,",
        "no pile of generated junk,",
        "no accidental architecture.",
        "docs/",
        "src/",
        "ui/",
        "build.rs",
        "Cargo.toml",
        ".gitignore",
        ".git/",
    ];

    let mut out = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();

        if bullet_items.contains(&trimmed) {
            out.push(format!("- {trimmed}"));
        } else {
            out.push(line.to_string());
        }
    }

    out.join("\n")
}

fn improve_common_wiki_links(input: &str) -> String {
    input
        .replace("See: docs/", "See: [`docs/`](Docs-Folder)")
        .replace("See: src/", "See: [`src/`](Source-Code-Folder)")
        .replace("See: ui/", "See: [`ui/`](UI-Folder)")
        .replace("See: build.rs", "See: [`build.rs`](Build-Script)")
        .replace("See: Cargo.toml", "See: [`Cargo.toml`](Cargo-Toml)")
        .replace("See: .gitignore", "See: [`.gitignore`](Gitignore)")
        .replace("See: .git/", "See: [`.git/`](Git-Metadata-Folder)")
}
