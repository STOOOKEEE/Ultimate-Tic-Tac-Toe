use crate::game::Move;

pub(crate) fn column_row_to_move(column: usize, row: usize) -> Option<Move> {
    if !(1..=9).contains(&column) || !(1..=9).contains(&row) {
        return None;
    }

    Some((column - 1, row - 1))
}

pub(crate) fn move_to_column_row((column, row): Move) -> (usize, usize) {
    (column + 1, row + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_conversion_uses_column_then_row() {
        assert_eq!(column_row_to_move(5, 8), Some((4, 7)));
        assert_eq!(move_to_column_row((4, 7)), (5, 8));
        assert_eq!(column_row_to_move(0, 8), None);
        assert_eq!(column_row_to_move(5, 10), None);
    }
}
