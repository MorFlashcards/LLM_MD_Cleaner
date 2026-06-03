use super::fences::{is_inside_fence, update_fence_state, Fence};
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
        } else {
            out.push(strip_html_from_line(line));
        }
    }

    out.join("\n")
}

fn strip_html_from_line(line: &str) -> String {
    let had_html = line.contains('<') && line.contains('>');
    let mut text = line.to_string();

    text = script_re().replace_all(&text, "").to_string();
    text = style_re().replace_all(&text, "").to_string();

    text = image_re()
        .replace_all(&text, |cap: &regex::Captures<'_>| {
            let tag = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            let src = html_attr(tag, "src").unwrap_or_default();
            let alt = html_attr(tag, "alt").unwrap_or_default();

            if src.is_empty() {
                String::new()
            } else {
                format!("![{alt}]({src})")
            }
        })
        .to_string();

    text = code_tag_re()
        .replace_all(&text, |cap: &regex::Captures<'_>| {
            let code = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();

            if code.is_empty() {
                String::new()
            } else {
                format!("`{}`", code.replace('`', r"\`"))
            }
        })
        .to_string();

    text = anchor_re()
        .replace_all(&text, |cap: &regex::Captures<'_>| {
            let tag = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            let href = html_attr(tag, "href").unwrap_or_default();
            let label = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();

            if href.is_empty() || label.is_empty() {
                label.to_string()
            } else if href == label {
                href
            } else {
                format!("[{label}]({href})")
            }
        })
        .to_string();

    text = block_tag_re().replace_all(&text, "\n").to_string();
    text = inline_tag_re().replace_all(&text, "").to_string();
    text = any_tag_re().replace_all(&text, "").to_string();

    if had_html {
        normalize_inline_spacing(&text)
    } else {
        text
    }
}

fn html_attr(tag: &str, name: &str) -> Option<String> {
    attr_re().captures_iter(tag).find_map(|cap| {
        let attr_name = cap.get(1)?.as_str();

        if !attr_name.eq_ignore_ascii_case(name) {
            return None;
        }

        cap.get(2)
            .or_else(|| cap.get(3))
            .or_else(|| cap.get(4))
            .map(|value| value.as_str().trim().to_string())
    })
}

fn normalize_inline_spacing(input: &str) -> String {
    let text = repeated_inline_space_re()
        .replace_all(input, " ")
        .to_string();

    space_before_punctuation_re()
        .replace_all(text.trim(), "$1")
        .to_string()
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

fn image_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)<\s*img\b[^>]*>").expect("valid image tag regex"))
}

fn code_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| {
        Regex::new(r"(?is)<\s*code\b[^>]*>(.*?)<\s*/\s*code\s*>").expect("valid inline code regex")
    })
}

fn anchor_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| {
        Regex::new(r"(?is)<\s*a\b[^>]*>(.*?)<\s*/\s*a\s*>").expect("valid anchor regex")
    })
}

fn attr_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| {
        Regex::new(r#"(?is)\b([a-z_:][-a-z0-9_:.]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#)
            .expect("valid HTML attribute regex")
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
        Regex::new(r"(?is)</?(span|strong|em|b|i|u|small|mark|kbd|samp|var)[^>]*>")
            .expect("valid inline tag regex")
    })
}

fn any_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)<[^>]+>").expect("valid catch-all tag regex"))
}

fn repeated_inline_space_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[ \t]{2,}").expect("valid repeated inline space regex"))
}

fn space_before_punctuation_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\s+([.,;:!?])").expect("valid punctuation spacing regex"))
}
