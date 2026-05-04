use crate::game::{Board, HeuristicParams, Move, Player, WIN_SCORE};
use crate::strong;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub(crate) const MAX_SEARCH_DEPTH: u32 = 24;

const MOVE_PRIORITY: [i32; 9] = [2, 1, 2, 1, 3, 1, 2, 1, 2];

const MATE_THRESHOLD: i32 = WIN_SCORE - 1000;

fn score_from_tt(stored: i32, ply: u32) -> i32 {
    if stored >= MATE_THRESHOLD {
        stored - ply as i32
    } else if stored <= -MATE_THRESHOLD {
        stored + ply as i32
    } else {
        stored
    }
}

fn score_to_tt(score: i32, ply: u32) -> i32 {
    if score >= MATE_THRESHOLD {
        score + ply as i32
    } else if score <= -MATE_THRESHOLD {
        score - ply as i32
    } else {
        score
    }
}

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
    killer_moves: Vec<[Option<Move>; 2]>,
    timed_out: bool,
}

impl SearchContext {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            tt: HashMap::with_capacity(capacity),
            killer_moves: vec![[None, None]; MAX_SEARCH_DEPTH as usize + 8],
            timed_out: false,
        }
    }

    fn killer_moves_at(&self, ply: u32) -> [Option<Move>; 2] {
        self.killer_moves
            .get(ply as usize)
            .copied()
            .unwrap_or([None, None])
    }

    fn record_killer(&mut self, ply: u32, mv: Move) {
        let Some(slot) = self.killer_moves.get_mut(ply as usize) else {
            return;
        };

        if slot[0] == Some(mv) {
            return;
        }

        slot[1] = slot[0];
        slot[0] = Some(mv);
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

#[derive(Clone, Copy)]
struct SearchNode {
    depth: u32,
    ply: u32,
    alpha: i32,
    beta: i32,
    is_max: bool,
    preferred_move: Option<Move>,
    extensions_left: u32,
}

pub(crate) struct SearchReport {
    pub(crate) best_move: Option<Move>,
    pub(crate) completed_depth: u32,
    pub(crate) elapsed: Duration,
    pub(crate) cache_size: usize,
}

fn minimax(
    board: &mut Board,
    mut node: SearchNode,
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

    let tt_entry = ctx.tt.get(&board.hash()).copied();
    if let Some(entry) = tt_entry {
        if entry.depth >= node.depth {
            let adjusted = score_from_tt(entry.score, node.ply);
            match entry.bound {
                Bound::Exact => {
                    return SearchOutcome {
                        score: adjusted,
                        best_move: entry.best_move,
                        completed: true,
                    };
                }
                Bound::Lower if adjusted >= node.beta => {
                    return SearchOutcome {
                        score: adjusted,
                        best_move: entry.best_move,
                        completed: true,
                    };
                }
                Bound::Upper if adjusted <= node.alpha => {
                    return SearchOutcome {
                        score: adjusted,
                        best_move: entry.best_move,
                        completed: true,
                    };
                }
                Bound::Lower | Bound::Upper => {}
            }
        }
    }

    if node.depth == 0
        && node.extensions_left > 0
        && !board.is_terminal()
        && (board.has_immediate_local_win_for_current_player()
            || board.has_immediate_local_win_for_opponent())
    {
        node.depth = 1;
    }

    if node.depth == 0 || board.is_terminal() {
        let raw = board.evaluate(config.params);
        let score = if raw >= MATE_THRESHOLD {
            raw - node.ply as i32
        } else if raw <= -MATE_THRESHOLD {
            raw + node.ply as i32
        } else {
            raw
        };
        return SearchOutcome {
            score,
            best_move: None,
            completed: true,
        };
    }

    let killers = ctx.killer_moves_at(node.ply);
    let mut moves = ordered_moves(
        board,
        tt_entry.and_then(|entry| entry.best_move),
        node.preferred_move,
        killers,
    );
    if moves.is_empty() {
        return SearchOutcome {
            score: board.evaluate(config.params),
            best_move: None,
            completed: true,
        };
    }

    let original_alpha = node.alpha;
    let original_beta = node.beta;
    let mut best_move = None;
    let mut best_score = if node.is_max { i32::MIN } else { i32::MAX };
    let mut completed = true;

    for candidate in moves.drain(..) {
        if config.expired() {
            ctx.timed_out = true;
            completed = false;
            break;
        }

        let Some(undo) = board.make_move_with_undo(candidate.0, candidate.1) else {
            continue;
        };

        let child = minimax(
            board,
            SearchNode {
                depth: node.depth - 1,
                ply: node.ply + 1,
                alpha: node.alpha,
                beta: node.beta,
                is_max: !node.is_max,
                preferred_move: None,
                extensions_left: node.extensions_left.saturating_sub(1),
            },
            config,
            ctx,
        );

        board.undo_move(undo);

        if !child.completed {
            completed = false;
            break;
        }

        if node.is_max {
            if child.score > best_score {
                best_score = child.score;
                best_move = Some(candidate);
            }
            node.alpha = node.alpha.max(best_score);
        } else {
            if child.score < best_score {
                best_score = child.score;
                best_move = Some(candidate);
            }
            node.beta = node.beta.min(best_score);
        }

        if node.beta <= node.alpha {
            ctx.record_killer(node.ply, candidate);
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
                depth: node.depth,
                score: score_to_tt(best_score, node.ply),
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

const MOVE_SCORE_MACRO_WIN: i32 = 1_000_000;
const MOVE_SCORE_MACRO_BLOCK: i32 = 650_000;
const MOVE_SCORE_TT: i32 = 500_000;
const MOVE_SCORE_PREVIOUS_BEST: i32 = 450_000;
const MOVE_SCORE_KILLER: i32 = 120_000;
const MOVE_SCORE_LOCAL_WIN: i32 = 5_000;
const MOVE_SCORE_SUICIDE: i32 = -100_000;
const MOVE_SCORE_MACRO_SUICIDE: i32 = -700_000;
const MOVE_SCORE_FREE_GIFT: i32 = -200;

fn score_candidate_move(
    board: &Board,
    mv: Move,
    tt_move: Option<Move>,
    preferred_move: Option<Move>,
    killer_moves: [Option<Move>; 2],
) -> i32 {
    let mut score = 0;

    if Some(mv) == tt_move {
        score += MOVE_SCORE_TT;
    }

    if Some(mv) == preferred_move {
        score += MOVE_SCORE_PREVIOUS_BEST;
    }

    if killer_moves.contains(&Some(mv)) {
        score += MOVE_SCORE_KILLER;
    }

    if board.would_complete_macro(mv) {
        score += MOVE_SCORE_MACRO_WIN;
    }

    if board.would_block_macro_threat(mv) {
        score += MOVE_SCORE_MACRO_BLOCK;
    }

    score += MOVE_PRIORITY[mv.1] * 10 + MOVE_PRIORITY[mv.0];

    if board.would_complete_local(mv) {
        score += MOVE_SCORE_LOCAL_WIN;
    }

    if board.move_opens_macro_win_for_opponent(mv) {
        score += MOVE_SCORE_MACRO_SUICIDE;
    }

    let opponent = board.current_player().opponent();
    match board.forced_target_after(mv) {
        Some(target) => {
            if board.player_can_win_local(target, opponent) {
                score += MOVE_SCORE_SUICIDE;
            }
        }
        None => {
            score += MOVE_SCORE_FREE_GIFT;
        }
    }

    score += board.move_tactical_importance(mv).clamp(-10_000, 10_000) / 10;
    score
}

fn ordered_moves(
    board: &Board,
    tt_move: Option<Move>,
    preferred_move: Option<Move>,
    killer_moves: [Option<Move>; 2],
) -> Vec<Move> {
    let mut moves = board.get_available_moves();
    moves.sort_by_key(|&mv| {
        Reverse(score_candidate_move(
            board,
            mv,
            tt_move,
            preferred_move,
            killer_moves,
        ))
    });
    moves
}

pub(crate) fn find_best_move(
    board: &Board,
    params: &HeuristicParams,
    time_limit: Duration,
    max_depth: u32,
) -> SearchReport {
    if max_depth >= MAX_SEARCH_DEPTH {
        if let Ok(Some(report)) = strong::find_best_move(board, time_limit) {
            if board.get_available_moves().contains(&report.best_move) {
                return SearchReport {
                    best_move: Some(report.best_move),
                    completed_depth: report.completed_depth,
                    elapsed: report.elapsed,
                    cache_size: 0,
                };
            }
        }
    }

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
    let mut root = board.clone();

    for depth in 1..=max_depth {
        let outcome = minimax(
            &mut root,
            SearchNode {
                depth,
                ply: 0,
                alpha: i32::MIN,
                beta: i32::MAX,
                is_max,
                preferred_move: best_move,
                extensions_left: 1,
            },
            &config,
            &mut ctx,
        );

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
