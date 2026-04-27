use crate::game::Move;

pub(crate) fn coordinates_to_move(column: usize, row: usize) -> Option<Move> {
    if !(1..=9).contains(&column) || !(1..=9).contains(&row) {
        return None;
    }

    let col = column - 1;
    let row = row - 1;
    Some(((row / 3) * 3 + (col / 3), (row % 3) * 3 + (col % 3)))
}

pub(crate) fn move_to_coordinates((macro_idx, micro_idx): Move) -> (usize, usize) {
    (
        (macro_idx % 3) * 3 + (micro_idx % 3) + 1,
        (macro_idx / 3) * 3 + (micro_idx / 3) + 1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_conversion_uses_column_then_row() {
        assert_eq!(coordinates_to_move(5, 8), Some((7, 4)));
        assert_eq!(move_to_coordinates((7, 4)), (5, 8));
        assert_eq!(coordinates_to_move(0, 8), None);
        assert_eq!(coordinates_to_move(5, 10), None);
    }
}
