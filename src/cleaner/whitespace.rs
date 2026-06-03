use regex::Regex;

pub(super) fn normalize(input: &str) -> String {
    let text = Regex::new(r"[ \t]+\n")
        .unwrap()
        .replace_all(input, "\n")
        .to_string();

    Regex::new(r"\n{3,}")
        .unwrap()
        .replace_all(&text, "\n\n")
        .trim()
        .to_string()
}
