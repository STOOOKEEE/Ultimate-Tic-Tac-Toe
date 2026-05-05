use crate::game::{Board, Move};
use crate::strong;
use std::time::{Duration, Instant};

pub(crate) struct SearchReport {
    pub(crate) best_move: Option<Move>,
    pub(crate) completed_depth: u32,
    pub(crate) elapsed: Duration,
}

pub(crate) fn find_best_move(board: &Board, time_limit: Duration) -> SearchReport {
    let start = Instant::now();

    match strong::find_best_move(board, time_limit) {
        Ok(Some(report)) if board.get_available_moves().contains(&report.best_move) => {
            SearchReport {
                best_move: Some(report.best_move),
                completed_depth: report.completed_depth,
                elapsed: report.elapsed,
            }
        }
        Ok(Some(_)) | Ok(None) | Err(_) => SearchReport {
            best_move: None,
            completed_depth: 0,
            elapsed: start.elapsed(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_best_move_uses_only_the_strong_engine_interface() {
        let board = Board::new();

        let report = find_best_move(&board, Duration::from_millis(50));

        assert!(
            report
                .best_move
                .is_some_and(|mv| board.get_available_moves().contains(&mv))
        );
    }
}
