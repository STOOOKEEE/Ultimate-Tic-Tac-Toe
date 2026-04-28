use rand::{RngExt, rng};
use std::fmt;
use std::sync::LazyLock;

pub(crate) type Move = (usize, usize);

pub(crate) const WIN_SCORE: i32 = 1_000_000;
const WIN_LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

#[derive(Clone, Copy, Debug)]
pub(crate) struct HeuristicParams {
    macro_win: i32,
    macro_two: i32,
    macro_one: i32,
    macro_fork: i32,
    macro_cell_weights: [i32; 9],
    center_macro_mult: f32,
    micro_two: i32,
    micro_one: i32,
    micro_center: i32,
    forced_board_threat: i32,
    mobility: i32,
}

impl Default for HeuristicParams {
    fn default() -> Self {
        Self {
            macro_win: 1_060,
            macro_two: 260,
            macro_one: 25,
            macro_fork: 420,
            macro_cell_weights: [34, 22, 34, 22, 48, 22, 34, 22, 34],
            center_macro_mult: 1.224_842_4,
            micro_two: 16,
            micro_one: 1,
            micro_center: 2,
            forced_board_threat: 500,
            mobility: 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Player {
    X,
    O,
}

impl Player {
    pub(crate) fn opponent(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Player::X => write!(f, "X"),
            Player::O => write!(f, "O"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CellState {
    Empty,
    Player(Player),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MacroState {
    Empty,
    Player(Player),
    Draw,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum GameOutcome {
    Ongoing,
    MacroWin(Player),
    TieBreakWin {
        winner: Player,
        x_boards: usize,
        o_boards: usize,
    },
    Draw {
        x_boards: usize,
        o_boards: usize,
    },
}

#[derive(Clone)]
pub(crate) struct Board {
    cells: [[CellState; 9]; 9],
    macros: [MacroState; 9],
    active_macro: Option<usize>,
    current_player: Player,
    hash: u64,
}

#[derive(Clone, Copy)]
pub(crate) struct MoveUndo {
    macro_idx: usize,
    micro_idx: usize,
    previous_macro: MacroState,
    previous_active_macro: Option<usize>,
    previous_player: Player,
    previous_hash: u64,
}

struct Zobrist {
    table: [[[u64; 2]; 9]; 9],
    player: u64,
    active_macro: [u64; 10],
}

static ZOBRIST: LazyLock<Zobrist> = LazyLock::new(|| {
    let mut r = rng();
    let mut table = [[[0_u64; 2]; 9]; 9];
    for macro_cells in &mut table {
        for cell in macro_cells {
            cell[0] = r.random();
            cell[1] = r.random();
        }
    }

    let mut active_macro = [0_u64; 10];
    for value in &mut active_macro {
        *value = r.random();
    }

    Zobrist {
        table,
        player: r.random(),
        active_macro,
    }
});

impl Board {
    pub(crate) fn new() -> Self {
        Self {
            cells: [[CellState::Empty; 9]; 9],
            macros: [MacroState::Empty; 9],
            active_macro: None,
            current_player: Player::X,
            hash: ZOBRIST.active_macro[9],
        }
    }

    pub(crate) fn current_player(&self) -> Player {
        self.current_player
    }

    pub(crate) fn hash(&self) -> u64 {
        self.hash
    }

    pub(crate) fn get_available_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(81);

        if let Some(macro_idx) = self.active_macro {
            if macro_idx >= 9 {
                return moves;
            }

            if self.macros[macro_idx] == MacroState::Empty {
                for micro_idx in 0..9 {
                    if self.cells[macro_idx][micro_idx] == CellState::Empty {
                        moves.push((macro_idx, micro_idx));
                    }
                }
                return moves;
            }
        }

        for macro_idx in 0..9 {
            if self.macros[macro_idx] == MacroState::Empty {
                for micro_idx in 0..9 {
                    if self.cells[macro_idx][micro_idx] == CellState::Empty {
                        moves.push((macro_idx, micro_idx));
                    }
                }
            }
        }

        moves
    }

    pub(crate) fn make_move(&mut self, macro_idx: usize, micro_idx: usize) -> bool {
        self.make_move_with_undo(macro_idx, micro_idx).is_some()
    }

    pub(crate) fn make_move_with_undo(
        &mut self,
        macro_idx: usize,
        micro_idx: usize,
    ) -> Option<MoveUndo> {
        if macro_idx >= 9 || micro_idx >= 9 {
            return None;
        }

        if self.macros[macro_idx] != MacroState::Empty {
            return None;
        }

        if self.cells[macro_idx][micro_idx] != CellState::Empty {
            return None;
        }

        if let Some(active) = self.active_macro {
            if active >= 9 {
                return None;
            }

            if active != macro_idx && self.macros[active] == MacroState::Empty {
                return None;
            }
        }

        let undo = MoveUndo {
            macro_idx,
            micro_idx,
            previous_macro: self.macros[macro_idx],
            previous_active_macro: self.active_macro,
            previous_player: self.current_player,
            previous_hash: self.hash,
        };

        let p_idx = player_index(self.current_player);
        self.hash ^= ZOBRIST.table[macro_idx][micro_idx][p_idx];
        self.cells[macro_idx][micro_idx] = CellState::Player(self.current_player);
        self.update_local_status(macro_idx);

        let old_active = self.active_macro.unwrap_or(9);
        self.active_macro = if self.macros[micro_idx] == MacroState::Empty {
            Some(micro_idx)
        } else {
            None
        };
        let new_active = self.active_macro.unwrap_or(9);

        self.hash ^=
            ZOBRIST.active_macro[old_active] ^ ZOBRIST.active_macro[new_active] ^ ZOBRIST.player;
        self.current_player = self.current_player.opponent();

        Some(undo)
    }

    pub(crate) fn undo_move(&mut self, undo: MoveUndo) {
        self.cells[undo.macro_idx][undo.micro_idx] = CellState::Empty;
        self.macros[undo.macro_idx] = undo.previous_macro;
        self.active_macro = undo.previous_active_macro;
        self.current_player = undo.previous_player;
        self.hash = undo.previous_hash;
    }

    pub(crate) fn outcome(&self) -> GameOutcome {
        if let Some(winner) = self.macro_winner() {
            return GameOutcome::MacroWin(winner);
        }

        if self.macros.iter().all(|&m| m != MacroState::Empty) {
            let (x_boards, o_boards) = self.local_board_counts();

            return match x_boards.cmp(&o_boards) {
                std::cmp::Ordering::Greater => GameOutcome::TieBreakWin {
                    winner: Player::X,
                    x_boards,
                    o_boards,
                },
                std::cmp::Ordering::Less => GameOutcome::TieBreakWin {
                    winner: Player::O,
                    x_boards,
                    o_boards,
                },
                std::cmp::Ordering::Equal => GameOutcome::Draw { x_boards, o_boards },
            };
        }

        GameOutcome::Ongoing
    }

    pub(crate) fn is_terminal(&self) -> bool {
        self.outcome() != GameOutcome::Ongoing
    }

    pub(crate) fn would_complete_local(&self, mv: Move) -> bool {
        let (macro_idx, micro_idx) = mv;
        if macro_idx >= 9 || micro_idx >= 9 {
            return false;
        }
        if self.macros[macro_idx] != MacroState::Empty {
            return false;
        }
        if self.cells[macro_idx][micro_idx] != CellState::Empty {
            return false;
        }
        let player_cell = CellState::Player(self.current_player);
        for line in &WIN_LINES {
            if !line.contains(&micro_idx) {
                continue;
            }
            let mut all_match = true;
            for &idx in line {
                if idx == micro_idx {
                    continue;
                }
                if self.cells[macro_idx][idx] != player_cell {
                    all_match = false;
                    break;
                }
            }
            if all_match {
                return true;
            }
        }
        false
    }

    pub(crate) fn would_complete_macro(&self, mv: Move) -> bool {
        let (macro_idx, _) = mv;
        if !self.would_complete_local(mv) {
            return false;
        }
        let player_macro = MacroState::Player(self.current_player);
        for line in &WIN_LINES {
            if !line.contains(&macro_idx) {
                continue;
            }
            let mut all_match = true;
            for &idx in line {
                if idx == macro_idx {
                    continue;
                }
                if self.macros[idx] != player_macro {
                    all_match = false;
                    break;
                }
            }
            if all_match {
                return true;
            }
        }
        false
    }

    pub(crate) fn would_block_macro_threat(&self, mv: Move) -> bool {
        let (macro_idx, _) = mv;
        if !self.would_complete_local(mv) {
            return false;
        }

        let opponent_macro = MacroState::Player(self.current_player.opponent());
        for line in &WIN_LINES {
            if !line.contains(&macro_idx) {
                continue;
            }

            let mut opponent_count = 0;
            let mut candidate_slot = false;
            for &idx in line {
                if idx == macro_idx {
                    candidate_slot = true;
                    continue;
                }
                if self.macros[idx] == opponent_macro {
                    opponent_count += 1;
                }
            }

            if candidate_slot && opponent_count == 2 {
                return true;
            }
        }

        false
    }

    pub(crate) fn player_can_win_local(&self, target: usize, player: Player) -> bool {
        if target >= 9 || self.macros[target] != MacroState::Empty {
            return false;
        }
        let player_cell = CellState::Player(player);
        let opponent_cell = CellState::Player(player.opponent());
        for line in &WIN_LINES {
            let mut player_count = 0;
            let mut empties = 0;
            let mut blocked = false;
            for &idx in line {
                let cell = self.cells[target][idx];
                if cell == player_cell {
                    player_count += 1;
                } else if cell == opponent_cell {
                    blocked = true;
                    break;
                } else {
                    empties += 1;
                }
            }
            if !blocked && player_count == 2 && empties == 1 {
                return true;
            }
        }
        false
    }

    pub(crate) fn forced_target_after(&self, mv: Move) -> Option<usize> {
        let (_, micro_idx) = mv;
        if micro_idx >= 9 {
            return None;
        }
        if self.macros[micro_idx] == MacroState::Empty {
            Some(micro_idx)
        } else {
            None
        }
    }

    pub(crate) fn move_opens_macro_win_for_opponent(&self, mv: Move) -> bool {
        let Some(target) = self.forced_target_after(mv) else {
            return false;
        };

        if !self.player_can_win_local(target, self.current_player.opponent()) {
            return false;
        }

        self.local_win_completes_macro_for(target, self.current_player.opponent())
    }

    pub(crate) fn move_tactical_importance(&self, mv: Move) -> i32 {
        let mut board = self.clone();
        if !board.make_move(mv.0, mv.1) {
            return 0;
        }

        let mobility = board.get_available_moves().len() as i32;
        let perspective = match self.current_player {
            Player::X => 1,
            Player::O => -1,
        };

        perspective * board.evaluate(&HeuristicParams::default()) - mobility
    }

    pub(crate) fn has_immediate_local_win_for_current_player(&self) -> bool {
        self.player_can_win_local_in_legal_moves(self.current_player)
    }

    pub(crate) fn has_immediate_local_win_for_opponent(&self) -> bool {
        self.player_can_win_local_in_legal_moves(self.current_player.opponent())
    }

    pub(crate) fn local_board_counts(&self) -> (usize, usize) {
        let mut x_boards = 0;
        let mut o_boards = 0;

        for macro_state in self.macros {
            match macro_state {
                MacroState::Player(Player::X) => x_boards += 1,
                MacroState::Player(Player::O) => o_boards += 1,
                MacroState::Empty | MacroState::Draw => {}
            }
        }

        (x_boards, o_boards)
    }

    pub(crate) fn evaluate(&self, params: &HeuristicParams) -> i32 {
        match self.outcome() {
            GameOutcome::MacroWin(Player::X) => return WIN_SCORE,
            GameOutcome::MacroWin(Player::O) => return -WIN_SCORE,
            GameOutcome::TieBreakWin {
                winner: Player::X, ..
            } => return WIN_SCORE,
            GameOutcome::TieBreakWin {
                winner: Player::O, ..
            } => return -WIN_SCORE,
            GameOutcome::Draw { .. } => return 0,
            GameOutcome::Ongoing => {}
        }

        let mut score = self.score_macro_lines(params);
        score += self.score_tie_break_pressure(params);

        if let Some(target) = self.active_macro {
            if target < 9 && self.macros[target] == MacroState::Empty {
                score += self.score_forced_board_threat(target, params);
            }
        }

        for macro_idx in 0..9 {
            let multiplier = if macro_idx == 4 {
                params.center_macro_mult
            } else {
                1.0
            };

            match self.macros[macro_idx] {
                MacroState::Player(Player::X) => {
                    score += ((params.macro_win + params.macro_cell_weights[macro_idx]) as f32
                        * multiplier) as i32;
                }
                MacroState::Player(Player::O) => {
                    score -= ((params.macro_win + params.macro_cell_weights[macro_idx]) as f32
                        * multiplier) as i32;
                }
                MacroState::Empty => {
                    let local_score = self.score_local_board(macro_idx, params);
                    score += ((local_score + params.macro_cell_weights[macro_idx] / 3) as f32
                        * multiplier) as i32;
                }
                MacroState::Draw => {}
            }
        }

        score
    }

    pub(crate) fn print(&self) {
        println!("\n    1 2 3   4 5 6   7 8 9 (colonnes)");
        match self.active_macro {
            Some(macro_idx) if macro_idx < 9 => {
                println!("    Grille imposee: {}", macro_idx + 1);
            }
            _ => println!("    Grille imposee: libre"),
        }

        for row in 0..9 {
            if row % 3 == 0 {
                println!("  +-------+-------+-------+");
            }

            print!("{} | ", row + 1);

            for col in 0..9 {
                let macro_idx = (row / 3) * 3 + (col / 3);
                let micro_idx = (row % 3) * 3 + (col % 3);

                match self.macros[macro_idx] {
                    MacroState::Player(Player::X) => print!("X "),
                    MacroState::Player(Player::O) => print!("O "),
                    MacroState::Draw => print!("- "),
                    MacroState::Empty => match self.cells[macro_idx][micro_idx] {
                        CellState::Player(Player::X) => print!("x "),
                        CellState::Player(Player::O) => print!("o "),
                        CellState::Empty => {
                            if self.active_macro.is_none() || self.active_macro == Some(macro_idx) {
                                print!(". ");
                            } else {
                                print!("  ");
                            }
                        }
                    },
                }

                if col % 3 == 2 {
                    print!("| ");
                }
            }

            println!();
        }

        println!("  +-------+-------+-------+ (lignes)\n");
    }

    fn update_local_status(&mut self, macro_idx: usize) {
        let grid = &self.cells[macro_idx];

        for line in &WIN_LINES {
            if grid[line[0]] != CellState::Empty
                && grid[line[0]] == grid[line[1]]
                && grid[line[1]] == grid[line[2]]
            {
                if let CellState::Player(player) = grid[line[0]] {
                    self.macros[macro_idx] = MacroState::Player(player);
                    return;
                }
            }
        }

        if grid.iter().all(|&cell| cell != CellState::Empty) {
            self.macros[macro_idx] = MacroState::Draw;
        }
    }

    fn macro_winner(&self) -> Option<Player> {
        for line in &WIN_LINES {
            if let MacroState::Player(player) = self.macros[line[0]] {
                if self.macros[line[1]] == MacroState::Player(player)
                    && self.macros[line[2]] == MacroState::Player(player)
                {
                    return Some(player);
                }
            }
        }

        None
    }

    fn local_win_completes_macro_for(&self, macro_idx: usize, player: Player) -> bool {
        if macro_idx >= 9 {
            return false;
        }

        let player_macro = MacroState::Player(player);
        for line in &WIN_LINES {
            if !line.contains(&macro_idx) {
                continue;
            }

            let mut count = 0;
            for &idx in line {
                if idx != macro_idx && self.macros[idx] == player_macro {
                    count += 1;
                }
            }

            if count == 2 {
                return true;
            }
        }

        false
    }

    fn player_can_win_local_in_legal_moves(&self, player: Player) -> bool {
        self.get_available_moves()
            .into_iter()
            .any(|(macro_idx, micro_idx)| self.move_wins_local_for(macro_idx, micro_idx, player))
    }

    fn move_wins_local_for(&self, macro_idx: usize, micro_idx: usize, player: Player) -> bool {
        if macro_idx >= 9 || micro_idx >= 9 {
            return false;
        }
        if self.macros[macro_idx] != MacroState::Empty {
            return false;
        }
        if self.cells[macro_idx][micro_idx] != CellState::Empty {
            return false;
        }

        let player_cell = CellState::Player(player);
        for line in &WIN_LINES {
            if !line.contains(&micro_idx) {
                continue;
            }

            let mut count = 0;
            for &idx in line {
                if idx != micro_idx && self.cells[macro_idx][idx] == player_cell {
                    count += 1;
                }
            }

            if count == 2 {
                return true;
            }
        }

        false
    }

    fn score_macro_lines(&self, params: &HeuristicParams) -> i32 {
        let mut score = 0;

        for line in &WIN_LINES {
            let mut x_count = 0;
            let mut o_count = 0;
            let mut blocked = false;

            for &idx in line {
                match self.macros[idx] {
                    MacroState::Player(Player::X) => x_count += 1,
                    MacroState::Player(Player::O) => o_count += 1,
                    MacroState::Draw => blocked = true,
                    MacroState::Empty => {}
                }
            }

            if blocked {
                continue;
            }

            if x_count > 0 && o_count == 0 {
                if x_count == 2 {
                    score += params.macro_two;
                } else {
                    score += params.macro_one;
                }
            } else if o_count > 0 && x_count == 0 {
                if o_count == 2 {
                    score -= params.macro_two;
                } else {
                    score -= params.macro_one;
                }
            }
        }

        score += self.score_macro_forks(params);
        score
    }

    fn score_macro_forks(&self, params: &HeuristicParams) -> i32 {
        let mut x_threat_targets = [0_u8; 9];
        let mut o_threat_targets = [0_u8; 9];

        for line in &WIN_LINES {
            let mut x_count = 0;
            let mut o_count = 0;
            let mut empty = None;
            let mut blocked = false;

            for &idx in line {
                match self.macros[idx] {
                    MacroState::Player(Player::X) => x_count += 1,
                    MacroState::Player(Player::O) => o_count += 1,
                    MacroState::Draw => blocked = true,
                    MacroState::Empty => empty = Some(idx),
                }
            }

            if blocked {
                continue;
            }

            if let Some(target) = empty {
                if x_count == 2 && o_count == 0 {
                    x_threat_targets[target] += 1;
                } else if o_count == 2 && x_count == 0 {
                    o_threat_targets[target] += 1;
                }
            }
        }

        let x_forks = x_threat_targets.iter().filter(|&&count| count >= 2).count() as i32;
        let o_forks = o_threat_targets.iter().filter(|&&count| count >= 2).count() as i32;

        (x_forks - o_forks) * params.macro_fork
    }

    fn score_tie_break_pressure(&self, params: &HeuristicParams) -> i32 {
        let (x_boards, o_boards) = self.local_board_counts();
        let board_delta = x_boards as i32 - o_boards as i32;
        let mobility_delta = self.mobility_for(Player::X) - self.mobility_for(Player::O);

        board_delta * params.macro_one + mobility_delta * params.mobility
    }

    fn mobility_for(&self, player: Player) -> i32 {
        let mut clone = self.clone();
        clone.current_player = player;
        clone.get_available_moves().len() as i32
    }

    fn score_local_board(&self, macro_idx: usize, params: &HeuristicParams) -> i32 {
        let mut score = 0;

        for line in &WIN_LINES {
            let mut x_count = 0;
            let mut o_count = 0;

            for &idx in line {
                match self.cells[macro_idx][idx] {
                    CellState::Player(Player::X) => x_count += 1,
                    CellState::Player(Player::O) => o_count += 1,
                    CellState::Empty => {}
                }
            }

            if x_count > 0 && o_count == 0 {
                score += if x_count == 2 {
                    params.micro_two
                } else {
                    params.micro_one
                };
            } else if o_count > 0 && x_count == 0 {
                score -= if o_count == 2 {
                    params.micro_two
                } else {
                    params.micro_one
                };
            }
        }

        match self.cells[macro_idx][4] {
            CellState::Player(Player::X) => score += params.micro_center,
            CellState::Player(Player::O) => score -= params.micro_center,
            CellState::Empty => {}
        }

        score
    }

    fn score_forced_board_threat(&self, target: usize, params: &HeuristicParams) -> i32 {
        let mut score = 0;

        for line in &WIN_LINES {
            let mut x_count = 0;
            let mut o_count = 0;
            let mut empties = 0;

            for &idx in line {
                match self.cells[target][idx] {
                    CellState::Player(Player::X) => x_count += 1,
                    CellState::Player(Player::O) => o_count += 1,
                    CellState::Empty => empties += 1,
                }
            }

            if empties == 0 {
                continue;
            }

            if self.current_player == Player::X && x_count == 2 && o_count == 0 {
                score += params.forced_board_threat;
            } else if self.current_player == Player::O && o_count == 2 && x_count == 0 {
                score -= params.forced_board_threat;
            }
        }

        score
    }
}

fn player_index(player: Player) -> usize {
    match player {
        Player::X => 0,
        Player::O => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_move_sets_next_forced_board() {
        let mut board = Board::new();

        assert!(board.make_move(0, 4));

        assert_eq!(board.active_macro, Some(4));
        assert_eq!(board.current_player, Player::O);
    }

    #[test]
    fn forced_board_restricts_available_moves() {
        let mut board = Board::new();
        assert!(board.make_move(0, 4));

        let moves = board.get_available_moves();

        assert_eq!(moves.len(), 9);
        assert!(moves.iter().all(|&(macro_idx, _)| macro_idx == 4));
        assert!(!board.make_move(0, 0));
    }

    #[test]
    fn completed_target_board_releases_next_player() {
        let mut board = Board::new();
        board.macros[4] = MacroState::Player(Player::X);

        assert!(board.make_move(0, 4));

        assert_eq!(board.active_macro, None);
    }

    #[test]
    fn invalid_indices_are_rejected() {
        let mut board = Board::new();

        assert!(!board.make_move(9, 0));
        assert!(!board.make_move(0, 9));
    }

    #[test]
    fn local_three_in_a_row_updates_macro_board() {
        let mut board = Board::new();
        board.cells[0][0] = CellState::Player(Player::X);
        board.cells[0][1] = CellState::Player(Player::X);
        board.cells[0][2] = CellState::Player(Player::X);

        board.update_local_status(0);

        assert_eq!(board.macros[0], MacroState::Player(Player::X));
    }

    #[test]
    fn undo_move_restores_board_state() {
        let mut board = Board::new();
        assert!(board.make_move(0, 4));

        let previous_cells = board.cells;
        let previous_macros = board.macros;
        let previous_active_macro = board.active_macro;
        let previous_player = board.current_player;
        let previous_hash = board.hash;

        let undo = board.make_move_with_undo(4, 0).expect("legal move");
        board.undo_move(undo);

        assert_eq!(board.cells, previous_cells);
        assert_eq!(board.macros, previous_macros);
        assert_eq!(board.active_macro, previous_active_macro);
        assert_eq!(board.current_player, previous_player);
        assert_eq!(board.hash, previous_hash);
    }

    #[test]
    fn macro_alignment_ends_game() {
        let mut board = Board::new();
        board.macros[0] = MacroState::Player(Player::X);
        board.macros[1] = MacroState::Player(Player::X);
        board.macros[2] = MacroState::Player(Player::X);

        assert!(board.is_terminal());
        assert_eq!(board.outcome(), GameOutcome::MacroWin(Player::X));
        assert_eq!(board.evaluate(&HeuristicParams::default()), WIN_SCORE);
    }

    #[test]
    fn full_board_uses_local_board_tie_break() {
        let mut board = Board::new();
        board.macros = [
            MacroState::Player(Player::X),
            MacroState::Player(Player::X),
            MacroState::Player(Player::O),
            MacroState::Player(Player::O),
            MacroState::Player(Player::O),
            MacroState::Player(Player::X),
            MacroState::Player(Player::X),
            MacroState::Player(Player::X),
            MacroState::Player(Player::O),
        ];

        assert!(board.is_terminal());
        assert_eq!(
            board.outcome(),
            GameOutcome::TieBreakWin {
                winner: Player::X,
                x_boards: 5,
                o_boards: 4,
            }
        );
        assert_eq!(board.evaluate(&HeuristicParams::default()), WIN_SCORE);
    }

    #[test]
    fn macro_two_parameter_scores_macro_threats() {
        let mut board = Board::new();
        board.macros[0] = MacroState::Player(Player::X);
        board.macros[1] = MacroState::Player(Player::X);

        let with_threat = board.evaluate(&HeuristicParams {
            macro_two: 500,
            ..HeuristicParams::default()
        });
        let without_threat = board.evaluate(&HeuristicParams {
            macro_two: 0,
            ..HeuristicParams::default()
        });

        assert!(with_threat > without_threat);
    }
}
