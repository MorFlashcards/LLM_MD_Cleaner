use super::fences::{is_inside_fence, update_fence_state, Fence};

pub(super) fn convert_tsv_tables(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut active_fence: Option<Fence> = None;
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if update_fence_state(&mut active_fence, trimmed) {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        }

        if is_inside_fence(active_fence) || !lines[i].contains('\t') {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        }

        let mut block = Vec::new();

        while i < lines.len() && lines[i].contains('\t') {
            block.push(lines[i]);
            i += 1;
        }

        if looks_like_tsv_table(&block) {
            out.extend(tsv_block_to_markdown_table(&block));
        } else {
            out.extend(block.into_iter().map(str::to_string));
        }
    }

    out.join("\n")
}

fn looks_like_tsv_table(block: &[&str]) -> bool {
    if block.len() < 2 {
        return false;
    }

    let first_col_count = block[0].split('\t').count();

    first_col_count >= 2
        && block
            .iter()
            .any(|row| row.split('\t').count() == first_col_count)
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
