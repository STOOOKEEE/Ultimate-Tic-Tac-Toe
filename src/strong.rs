use crate::{
    core::TicTacToe,
    game::{Board, Move},
    movegen::generate_moves,
    network::Network,
    search::{Search, endgame_trigger},
};
use std::io;
use std::path::Path;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

const DEFAULT_WEIGHTS: &str = "databin/gen160_weights.bin";

static DEFAULT_NETWORK: LazyLock<Option<Box<Network>>> = LazyLock::new(|| {
    if Path::new(DEFAULT_WEIGHTS).is_file() {
        Some(Network::load(DEFAULT_WEIGHTS.to_string()))
    } else {
        None
    }
});

pub(crate) fn move_to_square((column, row): Move) -> Option<u8> {
    if column >= 9 || row >= 9 {
        return None;
    }

    Some((row * 9 + column) as u8)
}

pub(crate) fn square_to_move(square: u8) -> Option<Move> {
    if square >= 81 {
        return None;
    }

    let row = square as usize / 9;
    let column = square as usize % 9;
    Some((column, row))
}

pub(crate) struct StrongSearchReport {
    pub(crate) best_move: Move,
    pub(crate) completed_depth: u32,
    pub(crate) elapsed: Duration,
}

pub(crate) struct StrongEngine {
    net: &'static Network,
    search: Search,
}

impl StrongEngine {
    pub(crate) fn load_default() -> io::Result<Self> {
        let net = DEFAULT_NETWORK.as_deref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("{DEFAULT_WEIGHTS} is missing"),
            )
        })?;

        Ok(Self {
            net,
            search: Search::new(),
        })
    }

    #[cfg(test)]
    pub(crate) fn best_square(&mut self, board: &TicTacToe, time_limit: Duration) -> u8 {
        let start = Instant::now();
        if endgame_trigger(board) {
            let exact_budget = time_limit.min(Duration::from_millis(500));
            if let Some((best, _score)) = self.search.think_exact(board, exact_budget) {
                return best;
            }
        }

        let remaining = time_limit
            .saturating_sub(start.elapsed())
            .max(Duration::from_millis(1));
        let (best, _depth) = self
            .search
            .iterative_deepening_think(board, self.net, remaining);
        best.unwrap_or_else(|| first_legal_square(board))
    }

    fn search_board(&mut self, board: &TicTacToe, time_limit: Duration) -> StrongSearchReport {
        let start = Instant::now();

        if endgame_trigger(board) {
            let exact_budget = time_limit.min(Duration::from_millis(500));
            if let Some((best_square, _score)) = self.search.think_exact(board, exact_budget)
                && let Some(best_move) = square_to_move(best_square)
            {
                return StrongSearchReport {
                    best_move,
                    completed_depth: 81_u32.saturating_sub(board.ply as u32),
                    elapsed: start.elapsed(),
                };
            }
        }

        let remaining = time_limit
            .saturating_sub(start.elapsed())
            .max(Duration::from_millis(1));
        let (best_square, completed_depth) = self
            .search
            .iterative_deepening_think(board, self.net, remaining);
        let best_square = best_square.unwrap_or_else(|| first_legal_square(board));

        StrongSearchReport {
            best_move: square_to_move(best_square).expect("engine move should map to Board move"),
            completed_depth: completed_depth.saturating_sub(1) as u32,
            elapsed: start.elapsed(),
        }
    }
}

pub(crate) fn find_best_move(
    board: &Board,
    time_limit: Duration,
) -> io::Result<Option<StrongSearchReport>> {
    let Some(engine_board) = board_to_engine_board(board) else {
        return Ok(None);
    };
    let mut engine = StrongEngine::load_default()?;

    Ok(Some(engine.search_board(&engine_board, time_limit)))
}

fn board_to_engine_board(board: &Board) -> Option<TicTacToe> {
    let mut engine_board = TicTacToe::new();

    for &mv in board.played_moves() {
        let square = move_to_square(mv)?;
        if engine_board.validate_move(square).is_err() {
            return None;
        }
        engine_board.make(square);
    }

    Some(engine_board)
}

fn first_legal_square(board: &TicTacToe) -> u8 {
    generate_moves(board).trailing_zeros() as u8
}

#[cfg(test)]
fn generate_move_list(board: &TicTacToe) -> Vec<u8> {
    let mut moves = generate_moves(board);
    let mut out = Vec::with_capacity(moves.count_ones() as usize);
    while moves != 0 {
        let mv = moves.trailing_zeros() as u8;
        out.push(mv);
        moves &= moves - 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_square_conversion_round_trips_column_row_coordinates() {
        assert_eq!(move_to_square((0, 0)), Some(0));
        assert_eq!(move_to_square((8, 0)), Some(8));
        assert_eq!(move_to_square((2, 2)), Some(20));
        assert_eq!(move_to_square((4, 4)), Some(40));
        assert_eq!(move_to_square((8, 8)), Some(80));
        assert_eq!(square_to_move(40), Some((4, 4)));

        for column in 0..9 {
            for row in 0..9 {
                let mv = (column, row);
                let square = move_to_square(mv).expect("move should map to square");
                assert_eq!(square_to_move(square), Some(mv));
            }
        }
    }

    #[test]
    fn strong_engine_legal_moves_match_current_board_after_played_moves() {
        let mut board = Board::new();
        let mut engine_board = TicTacToe::new();

        for mv in [(4, 4), (3, 3), (0, 1), (0, 5), (2, 6)] {
            assert!(board.make_move(mv.0, mv.1));
            engine_board.make(move_to_square(mv).expect("valid square"));

            let mut current_moves: Vec<u8> = board
                .get_available_moves()
                .into_iter()
                .map(|mv| move_to_square(mv).expect("valid square"))
                .collect();
            current_moves.sort_unstable();

            let mut engine_moves = generate_move_list(&engine_board);
            engine_moves.sort_unstable();

            assert_eq!(engine_moves, current_moves);
        }
    }

    #[test]
    fn strong_engine_loads_recovered_weights_and_selects_legal_opening() {
        let board = TicTacToe::new();
        let mut engine = StrongEngine::load_default().expect("gen160 weights should load");

        let best_square = engine.best_square(&board, std::time::Duration::from_millis(50));
        assert!(
            generate_move_list(&board).contains(&best_square),
            "engine returned an illegal square {best_square}"
        );
    }
}
