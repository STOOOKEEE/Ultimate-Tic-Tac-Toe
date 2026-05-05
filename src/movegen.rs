use crate::{
    constants::{CELL_TO_SUBBOARD_INDEX, MAP, WINDOW},
    core::TicTacToe,
};

/// Generate all moves and return them into another bitboard.
pub fn generate_moves(board: &TicTacToe) -> u128 {
    let occupied = board.bitboard;

    let mask = if let Some(current_focus) = board.current_focus
        && ((board.full_subboard | board.all_clear) & (1 << board.current_focus.unwrap())) == 0
    {
        WINDOW << MAP[current_focus as usize]
    } else {
        (0..81u8)
            .filter(|&i| {
                let sb = CELL_TO_SUBBOARD_INDEX[i as usize];
                (board.full_subboard | board.all_clear) & (1 << sb) == 0
            })
            .fold(0u128, |acc, i| acc | (1 << i))
    };

    mask & !occupied
}

#[cfg(test)]
mod tests {
    use super::*;

    fn move_count(moves: u128) -> u32 {
        moves.count_ones()
    }

    fn contains_subboard(moves: u128, subboard: u8) -> bool {
        (0..81u8).any(|square| {
            (moves & (1_u128 << square)) != 0 && CELL_TO_SUBBOARD_INDEX[square as usize] == subboard
        })
    }

    #[test]
    fn won_forced_board_releases_to_other_playable_boards() {
        let mut board = TicTacToe::new();
        board.current_focus = Some(4);
        board.all_clear = 1 << 4;

        let moves = generate_moves(&board);

        assert_eq!(move_count(moves), 72);
        assert!(!contains_subboard(moves, 4));
    }

    #[test]
    fn full_forced_board_releases_to_other_playable_boards() {
        let mut board = TicTacToe::new();
        board.current_focus = Some(4);
        board.full_subboard = 1 << 4;

        let moves = generate_moves(&board);

        assert_eq!(move_count(moves), 72);
        assert!(!contains_subboard(moves, 4));
    }
}
