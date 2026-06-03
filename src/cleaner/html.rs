use regex::Regex;
use std::sync::OnceLock;

pub(super) fn prefer_markdown_tail(input: &str) -> String {
    for marker in ["</pre>", "</div>"] {
        if let Some(index) = input.rfind(marker) {
            let tail = input[index + marker.len()..].trim();

            if looks_like_markdown_start(tail) {
                return tail.to_string();
            }
        }
    }

    input.to_string()
}

pub(super) fn decode_entities_and_breaks(input: &str) -> String {
    let decoded = html_escape::decode_html_entities(input).to_string();

    br_re().replace_all(&decoded, "\n").to_string()
}

pub(super) fn extract_code_block_text(input: &str) -> String {
    if input.contains("cm-content") && input.contains("<span") {
        let mut out = String::new();

        for cap in span_re().captures_iter(input) {
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

pub(super) fn strip_html_tags(input: &str) -> String {
    let mut text = input.to_string();

    text = script_re().replace_all(&text, "").to_string();
    text = style_re().replace_all(&text, "").to_string();

    // Preserve simple links before removing their tags.
    text = anchor_re()
        .replace_all(&text, |cap: &regex::Captures<'_>| {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let label = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();

            if href.is_empty() || label.is_empty() {
                label.to_string()
            } else if href == label {
                href.to_string()
            } else {
                format!("[{label}]({href})")
            }
        })
        .to_string();

    // Block-ish tags should break lines.
    text = block_tag_re().replace_all(&text, "\n").to_string();

    // Inline-ish tags should disappear without injecting line breaks into prose.
    text = inline_tag_re().replace_all(&text, "").to_string();

    any_tag_re().replace_all(&text, "").to_string()
}

fn looks_like_markdown_start(input: &str) -> bool {
    let trimmed = input.trim_start();

    trimmed.starts_with("# ")
        || trimmed.starts_with("## ")
        || trimmed.starts_with("```")
        || trimmed.starts_with("~~~")
        || trimmed.starts_with("- ")
}

fn br_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)<br\s*/?>").expect("valid br tag regex"))
}

fn span_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)<span[^>]*>(.*?)</span>").expect("valid span regex"))
}

fn script_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)<script.*?</script>").expect("valid script regex"))
}

fn style_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)<style.*?</style>").expect("valid style regex"))
}

fn anchor_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| {
        Regex::new(r#"(?is)<a\s+[^>]*href=["']([^"']+)["'][^>]*>(.*?)</a>"#)
            .expect("valid anchor regex")
    })
}

fn block_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| {
        Regex::new(
            r"(?is)</?(h1|h2|h3|h4|h5|h6|p|div|li|ul|ol|pre|table|tbody|thead|tr|td|th|blockquote|section|article|header|footer)[^>]*>",
        )
        .expect("valid block tag regex")
    })
}

fn inline_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| {
        Regex::new(r"(?is)</?(code|span|strong|em|b|i|u|small|mark|kbd|samp|var)[^>]*>")
            .expect("valid inline tag regex")
    })
}

fn any_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)<[^>]+>").expect("valid catch-all tag regex"))
}
