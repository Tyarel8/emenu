use eframe::{
    egui::{TextBuffer, TextFormat},
    epaint::{text::LayoutJob, Color32, FontId},
};

pub fn create_layout(
    input: &str,
    match_str: &str,
    pointer: &str,
    marker: &str,
    max_characters: usize,
    font_id: FontId,
) -> LayoutJob {
    let mut layout = LayoutJob::default();
    let ellipsis = "…";
    let default_color = Color32::GRAY;
    let highlight_color = Color32::LIGHT_GREEN;

    let pointer_len = pointer.chars().count();
    let marker_len = marker.chars().count();
    let match_str_len = match_str.chars().count();

    let max_characters = max_characters - pointer_len - marker_len;

    let mut start_idx = 0;
    let mut end_idx = match_str_len.min(max_characters);

    let mut highlight_indices = fuzzy_search_highlight(input, match_str);

    // figure out start and end indices
    let keep_chars_on_right = 5;
    if match_str_len > max_characters && !highlight_indices.is_empty() {
        let last_highlight = highlight_indices.iter().max().unwrap();

        if last_highlight > &(max_characters - keep_chars_on_right) {
            end_idx = (last_highlight + keep_chars_on_right).min(match_str_len);
            start_idx = end_idx - max_characters;
        }
    }

    // dbg!(max_characters);
    // dbg!(start_idx);
    // dbg!(end_idx);
    // dbg!(&highlight_indices);

    let start_ellipsis = start_idx > 0;
    let end_ellipsis = end_idx < match_str_len;

    if start_ellipsis {
        start_idx += 1;
    }

    if end_ellipsis {
        end_idx -= 1;
    }

    // add pointer and marker
    layout.append(
        pointer,
        0.0,
        TextFormat::simple(font_id.clone(), default_color),
    );
    layout.append(
        marker,
        0.0,
        TextFormat::simple(font_id.clone(), default_color),
    );

    if start_ellipsis {
        layout.append(
            ellipsis,
            0.0,
            TextFormat::simple(font_id.clone(), default_color),
        );
    }

    // Middle text

    let mut cur_idx = start_idx;
    highlight_indices.sort_unstable();
    for next_highlight in highlight_indices.into_iter().filter(|x| x >= &start_idx) {
        if cur_idx < next_highlight {
            layout.append(
                match_str.char_range(cur_idx..next_highlight),
                0.0,
                TextFormat::simple(font_id.clone(), default_color),
            );
        }
        cur_idx += (cur_idx..next_highlight).len();

        layout.append(
            match_str.char_range(cur_idx..cur_idx + 1),
            0.0,
            TextFormat::simple(font_id.clone(), highlight_color),
        );

        cur_idx += 1;
    }

    if cur_idx < end_idx {
        layout.append(
            match_str.char_range(cur_idx..end_idx),
            0.0,
            TextFormat::simple(font_id.clone(), default_color),
        );
    }

    if end_ellipsis {
        layout.append(ellipsis, 0.0, TextFormat::simple(font_id, default_color));
    }

    layout
}

// TODO: fix this, should split input in words and match better like fzf
fn fuzzy_search_highlight(search: &str, match_str: &str) -> Vec<usize> {
    let search_words = search.trim().to_lowercase();
    let match_str = match_str.to_lowercase();

    let mut indices = Vec::new();
    let mut start_index;

    for s_word in search_words.split_whitespace() {
        start_index = 0;

        for search_char in s_word.chars() {
            loop {
                if let Some(index) = match_str
                    .char_range(start_index..usize::MAX)
                    .chars()
                    .position(|x| x == search_char)
                {
                    if indices.contains(&(start_index + index)) {
                        start_index += index + 1;
                        continue;
                    }

                    indices.push(start_index + index);
                    start_index += index + 1;
                    break;
                }

                break;
            }
        }
    }

    indices
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("", "", vec![])]
    #[case("ss", "some", vec![0])]
    #[case("aihh", "There shall be neither light nor safety", vec![8, 17, 19, 26])]
    #[case("ulul", "En un lugar de la Mancha", vec![3, 6, 7, 15])]
    #[case("xyz", "hello world", vec![])]
    #[case("a", "", vec![])]
    #[case("hello", "hello", vec![0, 1, 2, 3, 4])]
    #[case("hlo", "hello", vec![0, 2, 4])]
    #[case("rust", "Rust is great", vec![0, 1, 2, 3])]
    #[case("abc", "aabbcc", vec![0, 2, 4])]
    #[case("test", "testing tests", vec![0, 1, 2, 3])]
    #[case("ing", "testing", vec![4, 5, 6])]
    #[case("x", "x", vec![0])]
    #[case("aa", "banana", vec![1, 3])]
    #[case("code", "The code is clean", vec![4, 5, 6, 7])]
    fn test_fuzzy_search_highlight(
        #[case] pattern: &str,
        #[case] text: &str,
        #[case] expected: Vec<usize>,
    ) {
        assert_eq!(
            fuzzy_search_highlight(pattern, text),
            expected,
            "Failed for pattern '{}' in text '{}'",
            pattern,
            text
        );
    }
}
