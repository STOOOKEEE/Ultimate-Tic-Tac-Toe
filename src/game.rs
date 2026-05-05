use rand::{RngExt, rng};
use std::fmt;
use std::sync::LazyLock;

/// Zero-based move coordinates in column, row order on the 9x9 board.
pub(crate) type Move = (usize, usize);

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

pub(crate) fn move_to_indices((column, row): Move) -> Option<(usize, usize)> {
    if column >= 9 || row >= 9 {
        return None;
    }

    Some(((row / 3) * 3 + (column / 3), (row % 3) * 3 + (column % 3)))
}

fn indices_to_move(macro_idx: usize, micro_idx: usize) -> Option<Move> {
    if macro_idx >= 9 || micro_idx >= 9 {
        return None;
    }

    Some((
        (macro_idx % 3) * 3 + (micro_idx % 3),
        (macro_idx / 3) * 3 + (micro_idx / 3),
    ))
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
    history: Vec<Move>,
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
            history: Vec::with_capacity(81),
        }
    }

    pub(crate) fn current_player(&self) -> Player {
        self.current_player
    }

    pub(crate) fn played_moves(&self) -> &[Move] {
        &self.history
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
                        moves.push(indices_to_move(macro_idx, micro_idx).expect("valid move"));
                    }
                }
                return moves;
            }
        }

        for macro_idx in 0..9 {
            if self.macros[macro_idx] == MacroState::Empty {
                for micro_idx in 0..9 {
                    if self.cells[macro_idx][micro_idx] == CellState::Empty {
                        moves.push(indices_to_move(macro_idx, micro_idx).expect("valid move"));
                    }
                }
            }
        }

        moves
    }

    pub(crate) fn make_move(&mut self, column: usize, row: usize) -> bool {
        if self.macro_winner().is_some() {
            return false;
        }

        let Some((macro_idx, micro_idx)) = move_to_indices((column, row)) else {
            return false;
        };

        if self.macros[macro_idx] != MacroState::Empty {
            return false;
        }

        if self.cells[macro_idx][micro_idx] != CellState::Empty {
            return false;
        }

        if let Some(active) = self.active_macro {
            if active >= 9 {
                return false;
            }

            if active != macro_idx && self.macros[active] == MacroState::Empty {
                return false;
            }
        }

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
        self.history.push((column, row));

        true
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
                && let CellState::Player(player) = grid[line[0]]
            {
                self.macros[macro_idx] = MacroState::Player(player);
                return;
            }
        }

        if grid.iter().all(|&cell| cell != CellState::Empty) {
            self.macros[macro_idx] = MacroState::Draw;
        }
    }

    fn macro_winner(&self) -> Option<Player> {
        for line in &WIN_LINES {
            if let MacroState::Player(player) = self.macros[line[0]]
                && self.macros[line[1]] == MacroState::Player(player)
                && self.macros[line[2]] == MacroState::Player(player)
            {
                return Some(player);
            }
        }

        None
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

        assert!(board.make_move(1, 1));

        assert_eq!(board.active_macro, Some(4));
        assert_eq!(board.current_player, Player::O);
    }

    #[test]
    fn forced_board_restricts_available_moves() {
        let mut board = Board::new();
        assert!(board.make_move(1, 1));

        let moves = board.get_available_moves();

        assert_eq!(moves.len(), 9);
        assert!(moves.iter().all(|&(column, row)| {
            let (macro_idx, _) = move_to_indices((column, row)).expect("valid move");
            macro_idx == 4
        }));
        assert!(!board.make_move(0, 0));
    }

    #[test]
    fn completed_target_board_releases_next_player() {
        let mut board = Board::new();
        board.macros[4] = MacroState::Player(Player::X);

        assert!(board.make_move(1, 1));

        assert_eq!(board.active_macro, None);
    }

    #[test]
    fn drawn_target_board_releases_next_player() {
        let mut board = Board::new();
        board.macros[4] = MacroState::Draw;

        assert!(board.make_move(1, 1));

        assert_eq!(board.active_macro, None);
    }

    #[test]
    fn decided_local_boards_are_not_playable() {
        let mut board = Board::new();
        board.macros[0] = MacroState::Player(Player::X);
        board.macros[4] = MacroState::Draw;

        let moves = board.get_available_moves();

        assert!(
            !moves
                .iter()
                .any(|&mv| move_to_indices(mv).is_some_and(|(macro_idx, _)| macro_idx == 0))
        );
        assert!(
            !moves
                .iter()
                .any(|&mv| move_to_indices(mv).is_some_and(|(macro_idx, _)| macro_idx == 4))
        );
        assert!(!board.make_move(0, 0));
        assert!(!board.make_move(4, 4));
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
    fn macro_alignment_ends_game() {
        let mut board = Board::new();
        board.macros[0] = MacroState::Player(Player::X);
        board.macros[1] = MacroState::Player(Player::X);
        board.macros[2] = MacroState::Player(Player::X);

        assert!(board.is_terminal());
        assert_eq!(board.outcome(), GameOutcome::MacroWin(Player::X));
    }

    #[test]
    fn macro_alignment_rejects_later_moves() {
        let mut board = Board::new();
        board.macros[0] = MacroState::Player(Player::X);
        board.macros[1] = MacroState::Player(Player::X);
        board.macros[2] = MacroState::Player(Player::X);

        assert!(!board.make_move(0, 3));
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
    }
}
