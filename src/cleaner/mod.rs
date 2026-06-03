mod chatter;
mod fences;
mod headings;
mod html;
mod lists;
mod tables;
mod whitespace;

/// Clean copied LLM Markdown without baking in project-specific document names.
///
/// The pipeline is intentionally ordered from broad extraction to Markdown repair:
/// copied HTML is normalized first, outer LLM wrappers are stripped next, and then
/// Markdown-aware passes repair fences, headings, tables, and lists.
pub fn clean_markdown(input: &str) -> String {
    let mut text = input.to_string();

    text = html::prefer_markdown_tail(&text);
    text = html::decode_entities_and_breaks(&text);
    text = html::extract_code_block_text(&text);
    text = html::strip_html_tags(&text);

    text = chatter::strip_llm_chatter(&text);
    text = chatter::remove_copy_code_artifacts(&text);
    text = fences::unwrap_outer_markdown_fence(&text);

    text = headings::fix_heading_jam(&text);
    text = fences::fix_unclosed_code_fences(&text);
    text = tables::convert_tsv_tables(&text);
    text = lists::fix_lists(&text);

    whitespace::normalize(&text)
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
    fn preserves_longer_fence_length() {
        let input = "````markdown\n```rust\nfn main() {}\n```\n````";
        assert_eq!(clean_markdown(input), "```rust\nfn main() {}\n```");
    }

    #[test]
    fn closes_longer_unclosed_fence_with_matching_length() {
        let input = "````text\n```inner fence is content";
        assert_eq!(
            clean_markdown(input),
            "````text\n```inner fence is content\n````"
        );
    }

    #[test]
    fn closes_tilde_fence_before_markdown_heading() {
        let input = "~~~bash\necho hello\n## Next Section\nText.";
        assert_eq!(
            clean_markdown(input),
            "~~~bash\necho hello\n~~~\n\n## Next Section\nText."
        );
    }

    #[test]
    fn strips_basic_llm_chatter() {
        let input = "Sure, here is the cleaned markdown:\n\n# Title\n\nBody\n\nLet me know if you need anything else.";
        assert_eq!(clean_markdown(input), "# Title\nBody");
    }

    #[test]
    fn strips_chatgpt_specific_shell_chatter() {
        let input = "Here is your markdown:\n\n# Title\nBody\n\nThanks for using ChatGPT!";
        assert_eq!(clean_markdown(input), "# Title\nBody");
    }

    #[test]
    fn removes_copy_code_artifacts_but_not_plain_copy_content() {
        let input = "Copy code\n```rust\nfn main() {}\n```\nCopied!\n\ncopy";
        assert_eq!(clean_markdown(input), "```rust\nfn main() {}\n```\ncopy");
    }

    #[test]
    fn unwraps_outer_markdown_fence() {
        let input = "```markdown\n# Title\nBody\n```";
        assert_eq!(clean_markdown(input), "# Title\nBody");
    }

    #[test]
    fn converts_multi_column_tsv_tables() {
        let input = "Name\tKind\tNote\nRust\tLanguage\tFast";
        assert_eq!(
            clean_markdown(input),
            "| Name | Kind | Note |\n|---|---|---|\n| Rust | Language | Fast |"
        );
    }

    #[test]
    fn does_not_convert_tsv_inside_code_fence() {
        let input = "```text\nName\tKind\nRust\tLanguage\n```";
        assert_eq!(clean_markdown(input), input);
    }

    #[test]
    fn fixes_jammed_bullet_markers() {
        let input = "-First item\n*Second item\n+Third item\n-[ ] Task item";
        assert_eq!(
            clean_markdown(input),
            "- First item\n* Second item\n+ Third item\n- [ ] Task item"
        );
    }

    #[test]
    fn leaves_negative_numbers_alone() {
        let input = "-42\n-3.14";
        assert_eq!(clean_markdown(input), "-42\n-3.14");
    }

    #[test]
    fn strips_inline_html_without_breaking_sentence() {
        let input = "Use <code>cargo test</code> before pushing.";
        assert_eq!(clean_markdown(input), "Use cargo test before pushing.");
    }

    #[test]
    fn converts_simple_html_links_to_markdown_links() {
        let input = r#"<p>See <a href="https://example.com">Example</a>.</p>"#;
        assert_eq!(clean_markdown(input), "See [Example](https://example.com).");
    }
}
