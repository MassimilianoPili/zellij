use crate::output::CharacterChunk;
use crate::panes::terminal_character::{
    AnsiCode, RcCharacterStyles, TerminalCharacter, RESET_STYLES,
};
use crate::tab::DropZone;
use zellij_utils::data::Style;

/// An outline over the part of a pane's content area where a pane dragged with the mouse goes if
/// it is dropped there: the half on that side, or the whole pane for a swap. Only the outline is
/// drawn, so the content of the pane stays visible.
pub fn drop_zone_overlay_chunks(
    content_x: usize,
    content_y: usize,
    content_columns: usize,
    content_rows: usize,
    zone: DropZone,
    style: &Style,
) -> Vec<CharacterChunk> {
    let (x, y, width, height) = match zone {
        DropZone::Left => (content_x, content_y, content_columns / 2, content_rows),
        DropZone::Right => (
            content_x + content_columns - content_columns / 2,
            content_y,
            content_columns / 2,
            content_rows,
        ),
        DropZone::Top => (content_x, content_y, content_columns, content_rows / 2),
        DropZone::Bottom => (
            content_x,
            content_y + content_rows - content_rows / 2,
            content_columns,
            content_rows / 2,
        ),
        DropZone::Center => (content_x, content_y, content_columns, content_rows),
    };
    if width < 2 || height < 2 {
        return vec![];
    }
    let outline_style: RcCharacterStyles = RESET_STYLES
        .foreground(Some(AnsiCode::from(style.colors.frame_highlight.emphasis_0)))
        .bold(Some(AnsiCode::On))
        .into();
    let character = |c: char| TerminalCharacter::new_styled(c, outline_style.clone());
    let label = match zone {
        DropZone::Left => " DOCK LEFT ",
        DropZone::Right => " DOCK RIGHT ",
        DropZone::Top => " DOCK ABOVE ",
        DropZone::Bottom => " DOCK BELOW ",
        DropZone::Center => " SWAP ",
    };

    let mut top_border = vec![character('┏')];
    let label_fits = width >= label.chars().count() + 4;
    let mut top_middle: Vec<char> = vec!['━'; width - 2];
    if label_fits {
        for (index, label_character) in label.chars().enumerate() {
            top_middle[index + 1] = label_character;
        }
    }
    top_border.extend(top_middle.into_iter().map(character));
    top_border.push(character('┓'));

    let mut bottom_border = vec![character('┗')];
    bottom_border.extend((0..width - 2).map(|_| character('━')));
    bottom_border.push(character('┛'));

    let mut chunks = vec![
        CharacterChunk::new(top_border, x, y),
        CharacterChunk::new(bottom_border, x, y + height - 1),
    ];
    for line in y + 1..y + height - 1 {
        chunks.push(CharacterChunk::new(vec![character('┃')], x, line));
        chunks.push(CharacterChunk::new(vec![character('┃')], x + width - 1, line));
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outline_cells(chunks: &[CharacterChunk]) -> Vec<(usize, usize, char)> {
        let mut cells = vec![];
        for chunk in chunks {
            for (index, terminal_character) in chunk.terminal_characters.iter().enumerate() {
                cells.push((chunk.x + index, chunk.y, terminal_character.character));
            }
        }
        cells.sort();
        cells
    }

    #[test]
    fn right_zone_outlines_the_right_half_of_the_content() {
        let chunks = drop_zone_overlay_chunks(10, 5, 20, 4, DropZone::Right, &Style::default());
        let cells = outline_cells(&chunks);
        assert!(cells.contains(&(20, 5, '┏')), "top left corner of the right half");
        assert!(cells.contains(&(29, 5, '┓')), "top right corner of the content");
        assert!(cells.contains(&(20, 8, '┗')), "bottom left corner");
        assert!(cells.contains(&(29, 8, '┛')), "bottom right corner");
        assert!(
            cells.iter().all(|(x, _, _)| *x >= 20 && *x <= 29),
            "nothing is drawn over the left half"
        );
    }

    #[test]
    fn center_zone_outlines_the_whole_content_with_its_label() {
        let chunks = drop_zone_overlay_chunks(0, 0, 30, 6, DropZone::Center, &Style::default());
        let top_row: String = chunks[0]
            .terminal_characters
            .iter()
            .map(|c| c.character)
            .collect();
        assert!(top_row.starts_with("┏━ SWAP ━"), "the label is on the top border");
        assert!(top_row.ends_with('┓'));
    }

    #[test]
    fn zone_too_small_for_an_outline_draws_nothing() {
        let chunks = drop_zone_overlay_chunks(0, 0, 3, 1, DropZone::Top, &Style::default());
        assert!(chunks.is_empty());
    }
}
