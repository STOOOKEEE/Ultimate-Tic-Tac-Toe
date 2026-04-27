use crate::game::{Board, HeuristicParams, Move, Player};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub(crate) const MAX_SEARCH_DEPTH: u32 = 24;

const MOVE_PRIORITY: [i32; 9] = [2, 1, 2, 1, 3, 1, 2, 1, 2];

#[derive(Clone, Copy)]
enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Clone, Copy)]
struct TransEntry {
    depth: u32,
    score: i32,
    best_move: Option<Move>,
    bound: Bound,
}

struct SearchContext {
    tt: HashMap<u64, TransEntry>,
    timed_out: bool,
}

impl SearchContext {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            tt: HashMap::with_capacity(capacity),
            timed_out: false,
        }
    }
}

struct SearchConfig<'a> {
    start: Instant,
    limit: Duration,
    params: &'a HeuristicParams,
}

impl SearchConfig<'_> {
    fn expired(&self) -> bool {
        self.start.elapsed() >= self.limit
    }
}

#[derive(Clone, Copy)]
struct SearchOutcome {
    score: i32,
    best_move: Option<Move>,
    completed: bool,
}

pub(crate) struct SearchReport {
    pub(crate) best_move: Option<Move>,
    pub(crate) completed_depth: u32,
    pub(crate) elapsed: Duration,
    pub(crate) cache_size: usize,
}

fn minimax(
    board: &Board,
    depth: u32,
    mut alpha: i32,
    mut beta: i32,
    is_max: bool,
    config: &SearchConfig<'_>,
    ctx: &mut SearchContext,
) -> SearchOutcome {
    if config.expired() {
        ctx.timed_out = true;
        return SearchOutcome {
            score: board.evaluate(config.params),
            best_move: None,
            completed: false,
        };
    }

    if let Some(entry) = ctx.tt.get(&board.hash()).copied() {
        if entry.depth >= depth {
            match entry.bound {
                Bound::Exact => {
                    return SearchOutcome {
                        score: entry.score,
                        best_move: entry.best_move,
                        completed: true,
                    };
                }
                Bound::Lower if entry.score >= beta => {
                    return SearchOutcome {
                        score: entry.score,
                        best_move: entry.best_move,
                        completed: true,
                    };
                }
                Bound::Upper if entry.score <= alpha => {
                    return SearchOutcome {
                        score: entry.score,
                        best_move: entry.best_move,
                        completed: true,
                    };
                }
                Bound::Lower | Bound::Upper => {}
            }
        }
    }

    if depth == 0 || board.is_terminal() {
        return SearchOutcome {
            score: board.evaluate(config.params),
            best_move: None,
            completed: true,
        };
    }

    let mut moves = ordered_moves(board);
    if moves.is_empty() {
        return SearchOutcome {
            score: board.evaluate(config.params),
            best_move: None,
            completed: true,
        };
    }

    let original_alpha = alpha;
    let original_beta = beta;
    let mut best_move = None;
    let mut best_score = if is_max { i32::MIN } else { i32::MAX };
    let mut completed = true;

    for candidate in moves.drain(..) {
        if config.expired() {
            ctx.timed_out = true;
            completed = false;
            break;
        }

        let mut next_board = board.clone();
        next_board.make_move(candidate.0, candidate.1);

        let child = minimax(&next_board, depth - 1, alpha, beta, !is_max, config, ctx);

        if !child.completed {
            completed = false;
            break;
        }

        if is_max {
            if child.score > best_score {
                best_score = child.score;
                best_move = Some(candidate);
            }
            alpha = alpha.max(best_score);
        } else {
            if child.score < best_score {
                best_score = child.score;
                best_move = Some(candidate);
            }
            beta = beta.min(best_score);
        }

        if beta <= alpha {
            break;
        }
    }

    if completed {
        let bound = if best_score <= original_alpha {
            Bound::Upper
        } else if best_score >= original_beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        ctx.tt.insert(
            board.hash(),
            TransEntry {
                depth,
                score: best_score,
                best_move,
                bound,
            },
        );
    }

    SearchOutcome {
        score: best_score,
        best_move,
        completed,
    }
}

fn ordered_moves(board: &Board) -> Vec<Move> {
    let mut moves = board.get_available_moves();
    moves.sort_by_key(|&(macro_idx, micro_idx)| {
        Reverse(MOVE_PRIORITY[micro_idx] * 10 + MOVE_PRIORITY[macro_idx])
    });
    moves
}

pub(crate) fn find_best_move(
    board: &Board,
    params: &HeuristicParams,
    time_limit: Duration,
    max_depth: u32,
) -> SearchReport {
    let start = Instant::now();
    let config = SearchConfig {
        start,
        limit: time_limit,
        params,
    };
    let mut ctx = SearchContext::with_capacity(200_000);
    let mut best_move = None;
    let mut completed_depth = 0;
    let is_max = board.current_player() == Player::X;

    for depth in 1..=max_depth {
        let outcome = minimax(board, depth, i32::MIN, i32::MAX, is_max, &config, &mut ctx);

        if !outcome.completed || ctx.timed_out {
            break;
        }

        if let Some(candidate) = outcome.best_move {
            best_move = Some(candidate);
            completed_depth = depth;
        }
    }

    if best_move.is_none() {
        best_move = board.get_available_moves().into_iter().next();
    }

    SearchReport {
        best_move,
        completed_depth,
        elapsed: start.elapsed(),
        cache_size: ctx.tt.len(),
    }
}
