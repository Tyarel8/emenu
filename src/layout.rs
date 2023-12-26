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

    let mut highlight_idices = fuzzy_search_highlight(input, match_str);

    // figure out start and end idices
    let keep_chars_on_right = 5;
    if match_str_len > max_characters && highlight_idices.len() > 0 {
        let last_highlight = highlight_idices.iter().max().unwrap();

        if last_highlight > &(max_characters - keep_chars_on_right) {
            end_idx = (last_highlight + keep_chars_on_right).min(match_str_len);
            start_idx = end_idx - max_characters;
        }
    }

    // dbg!(max_characters);
    // dbg!(start_idx);
    // dbg!(end_idx);
    // dbg!(&highlight_idices);

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
    highlight_idices.sort_unstable();
    for next_highlight in highlight_idices.into_iter().filter(|x| x >= &start_idx) {
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

    layout.append(
        match_str.char_range(cur_idx..end_idx),
        0.0,
        TextFormat::simple(font_id.clone(), default_color),
    );

    if end_ellipsis {
        layout.append(ellipsis, 0.0, TextFormat::simple(font_id, default_color));
    }

    layout
}

// TODO: fix this, should split input in words and match better like fzf
fn fuzzy_search_highlight(search: &str, match_str: &str) -> Vec<usize> {
    let search = search.replace(' ', "");
    let mut indices = Vec::new();
    let mut start_index;

    for search_char in search.chars() {
        start_index = 0;

        loop {
            if let Some(index) = match_str[start_index..].find(search_char) {
                if indices.contains(&(start_index + index)) {
                    start_index += index + 1;
                    continue;
                }

                indices.push(start_index + index);
                break;
            }

            break;
        }
    }

    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_search_highlight() {
        assert_eq!(fuzzy_search_highlight("", ""), vec![]);

        assert_eq!(
            fuzzy_search_highlight("aihh", "There shall be neither light nor safety"),
            vec![8, 17, 1, 7]
        );

        assert_eq!(
            fuzzy_search_highlight("ulul", "En un lugar de la Mancha"),
            vec![3, 6, 7, 15]
        );
    }
}
