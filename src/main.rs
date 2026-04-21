use std::io::{self, Write};
use std::time::{Instant, Duration};
use rand::{rng, RngExt};
use std::env;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct HeuristicParams {
    pub macro_win: i32,
    pub center_macro_mult: f32,
    pub micro_two: i32,
    pub micro_one: i32,
    pub micro_center: i32,
    pub macro_two: i32,
}

impl Default for HeuristicParams {
    fn default() -> Self {
        Self {
            macro_win: 1060,
            center_macro_mult: 1.2248424,
            micro_two: 16,
            micro_one: 0,
            micro_center: 1,
            macro_two: 100,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player { X, O }

impl Player {
    fn opponent(self) -> Self {
        match self { Player::X => Player::O, Player::O => Player::X }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellState { Empty, Player(Player) }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MacroState { Empty, Player(Player), Draw }

#[derive(Clone)]
pub struct Board {
    pub cells: [[CellState; 9]; 9],
    pub macros: [MacroState; 9],
    pub active_macro: Option<usize>,
    pub current_player: Player,
    pub hash: u64,
}

struct Zobrist {
    table: [[[u64; 2]; 9]; 9],
    player: u64,
    active_macro: [u64; 10],
}

lazy_static::lazy_static! {
    static ref ZOBRIST: Zobrist = {
        let mut r = rng();
        let mut table = [[[0u64; 2]; 9]; 9];
        for i in 0..9 {
            for j in 0..9 {
                table[i][j][0] = r.random();
                table[i][j][1] = r.random();
            }
        }
        let mut active_macro = [0u64; 10];
        for i in 0..10 { active_macro[i] = r.random(); }
        Zobrist { table, player: r.random(), active_macro }
    };
}

const WIN_LINES: [[usize; 3]; 8] = [
    [0, 1, 2], [3, 4, 5], [6, 7, 8],
    [0, 3, 6], [1, 4, 7], [2, 5, 8],
    [0, 4, 8], [2, 4, 6]
];

const MOVE_PRIORITY: [i32; 9] = [2, 1, 2, 1, 3, 1, 2, 1, 2];

impl Board {
    pub fn new() -> Self {
        let mut b = Self {
            cells: [[CellState::Empty; 9]; 9],
            macros: [MacroState::Empty; 9],
            active_macro: None,
            current_player: Player::X,
            hash: 0,
        };
        b.hash = ZOBRIST.active_macro[9];
        b
    }

    pub fn get_available_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::with_capacity(81);
        if let Some(m) = self.active_macro {
            if self.macros[m] == MacroState::Empty {
                for i in 0..9 {
                    if self.cells[m][i] == CellState::Empty { moves.push((m, i)); }
                }
                return moves;
            }
        }
        for m in 0..9 {
            if self.macros[m] == MacroState::Empty {
                for i in 0..9 {
                    if self.cells[m][i] == CellState::Empty { moves.push((m, i)); }
                }
            }
        }
        moves
    }

    pub fn make_move(&mut self, macro_idx: usize, micro_idx: usize) -> bool {
        if self.macros[macro_idx] != MacroState::Empty { return false; }
        if self.cells[macro_idx][micro_idx] != CellState::Empty { return false; }
        if let Some(active) = self.active_macro {
            if active != macro_idx && self.macros[active] == MacroState::Empty { return false; }
        }

        let p_idx = if self.current_player == Player::X { 0 } else { 1 };
        self.hash ^= ZOBRIST.table[macro_idx][micro_idx][p_idx];
        self.cells[macro_idx][micro_idx] = CellState::Player(self.current_player);
        self.check_macro_win(macro_idx);

        let old_active = self.active_macro.unwrap_or(9);
        self.active_macro = if self.macros[micro_idx] == MacroState::Empty { Some(micro_idx) } else { None };
        let new_active = self.active_macro.unwrap_or(9);
        
        self.hash ^= ZOBRIST.active_macro[old_active] ^ ZOBRIST.active_macro[new_active] ^ ZOBRIST.player;
        self.current_player = self.current_player.opponent();
        true
    }

    fn check_macro_win(&mut self, macro_idx: usize) {
        let grid = &self.cells[macro_idx];
        for line in WIN_LINES.iter() {
            if grid[line[0]] != CellState::Empty && grid[line[0]] == grid[line[1]] && grid[line[1]] == grid[line[2]] {
                if let CellState::Player(p) = grid[line[0]] {
                    self.macros[macro_idx] = MacroState::Player(p);
                    return;
                }
            }
        }
        if grid.iter().all(|&c| c != CellState::Empty) { self.macros[macro_idx] = MacroState::Draw; }
    }

    pub fn is_terminal(&self) -> bool {
        for line in WIN_LINES.iter() {
            if self.macros[line[0]] != MacroState::Empty && self.macros[line[0]] != MacroState::Draw &&
               self.macros[line[0]] == self.macros[line[1]] && self.macros[line[1]] == self.macros[line[2]] {
                return true;
            }
        }
        self.macros.iter().all(|&m| m != MacroState::Empty)
    }

    pub fn evaluate(&self, params: &HeuristicParams) -> i32 {
        let mut score = 0;
        for line in WIN_LINES.iter() {
            if self.macros[line[0]] != MacroState::Empty && self.macros[line[0]] != MacroState::Draw &&
               self.macros[line[0]] == self.macros[line[1]] && self.macros[line[1]] == self.macros[line[2]] {
                if let MacroState::Player(p) = self.macros[line[0]] {
                    return if p == Player::X { 1000000 } else { -1000000 };
                }
            }
        }
        if self.macros.iter().all(|&m| m != MacroState::Empty) {
            let (mut x, mut o) = (0, 0);
            for m in self.macros.iter() {
                if *m == MacroState::Player(Player::X) { x += 1; }
                else if *m == MacroState::Player(Player::O) { o += 1; }
            }
            return if x > o { 1000000 } else if o > x { -1000000 } else { 0 };
        }

        if let Some(target) = self.active_macro {
            let grid = &self.cells[target];
            for line in WIN_LINES.iter() {
                let (mut x, mut o) = (0, 0);
                for &idx in line.iter() {
                    match grid[idx] { CellState::Player(Player::X) => x += 1, CellState::Player(Player::O) => o += 1, _ => {} }
                }
                if self.current_player == Player::O && x == 2 && o == 0 { score += 150; }
                else if self.current_player == Player::X && o == 2 && x == 0 { score -= 150; }
            }
        }

        for m in 0..9 {
            let mult = if m == 4 { params.center_macro_mult } else { 1.0 };
            match self.macros[m] {
                MacroState::Player(Player::X) => score += (params.macro_win as f32 * mult) as i32,
                MacroState::Player(Player::O) => score -= (params.macro_win as f32 * mult) as i32,
                _ => {
                    let mut s = 0;
                    for line in WIN_LINES.iter() {
                        let (mut x, mut o) = (0, 0);
                        for &idx in line.iter() {
                            match self.cells[m][idx] { CellState::Player(Player::X) => x += 1, CellState::Player(Player::O) => o += 1, _ => {} }
                        }
                        if x > 0 && o == 0 { s += if x == 2 { params.micro_two } else { params.micro_one }; }
                        else if o > 0 && x == 0 { s -= if o == 2 { params.micro_two } else { params.micro_one }; }
                    }
                    if self.cells[m][4] == CellState::Player(Player::X) { s += params.micro_center; }
                    else if self.cells[m][4] == CellState::Player(Player::O) { s -= params.micro_center; }
                    score += (s as f32 * mult) as i32;
                }
            }
        }
        score
    }

    pub fn print(&self) {
        println!("\n    1 2 3   4 5 6   7 8 9 (Colonnes)");
        for row in 0..9 {
            if row % 3 == 0 { println!("  +-------+-------+-------+"); }
            print!("{} | ", row + 1);
            for col in 0..9 {
                let (m, u) = ((row/3)*3 + (col/3), (row%3)*3 + (col%3));
                if self.macros[m] == MacroState::Player(Player::X) { print!("X "); }
                else if self.macros[m] == MacroState::Player(Player::O) { print!("O "); }
                else if self.macros[m] == MacroState::Draw { print!("- "); }
                else {
                    match self.cells[m][u] {
                        CellState::Player(Player::X) => print!("x "),
                        CellState::Player(Player::O) => print!("o "),
                        _ => {
                            if Some(m) == self.active_macro || self.active_macro.is_none() { print!(". "); }
                            else { print!("  "); }
                        }
                    }
                }
                if col % 3 == 2 { print!("| "); }
            }
            println!("");
        }
        println!("  +-------+-------+-------+ (Lignes)\n");
    }
}

#[derive(Clone, Copy)]
struct TransEntry { depth: u32, score: i32 }
pub struct SearchContext { tt: HashMap<u64, TransEntry> }

pub fn minimax(board: &Board, depth: u32, mut alpha: i32, mut beta: i32, is_max: bool, start: Instant, limit: Duration, params: &HeuristicParams, ctx: &mut SearchContext) -> (i32, Option<(usize, usize)>) {
    if let Some(entry) = ctx.tt.get(&board.hash) { if entry.depth >= depth { return (entry.score, None); } }
    if depth == 0 || board.is_terminal() || start.elapsed() >= limit { return (board.evaluate(params), None); }
    let mut moves = board.get_available_moves();
    if moves.is_empty() { return (board.evaluate(params), None); }
    moves.sort_by_key(|&(_, u)| std::cmp::Reverse(MOVE_PRIORITY[u]));
    let mut best_m = None;
    if is_max {
        let mut max_e = i32::MIN;
        for m in moves {
            let mut nb = board.clone(); nb.make_move(m.0, m.1);
            let (e, _) = minimax(&nb, depth-1, alpha, beta, false, start, limit, params, ctx);
            let s = e + if depth > 1 { (m.1 % 3) as i32 } else { 0 };
            if s > max_e { max_e = s; best_m = Some(m); }
            alpha = alpha.max(s);
            if beta <= alpha || start.elapsed() >= limit { break; }
        }
        ctx.tt.insert(board.hash, TransEntry { depth, score: max_e });
        (max_e, best_m)
    } else {
        let mut min_e = i32::MAX;
        for m in moves {
            let mut nb = board.clone(); nb.make_move(m.0, m.1);
            let (e, _) = minimax(&nb, depth-1, alpha, beta, true, start, limit, params, ctx);
            let s = e - if depth > 1 { (m.1 % 3) as i32 } else { 0 };
            if s < min_e { min_e = s; best_m = Some(m); }
            beta = beta.min(s);
            if beta <= alpha || start.elapsed() >= limit { break; }
        }
        ctx.tt.insert(board.hash, TransEntry { depth, score: min_e });
        (min_e, best_m)
    }
}

fn main() {
    println!("=== ULTIMATE TIC TAC TOE - IA PRO ===");
    print!("Qui commence ? (1: IA (X), 2: JOUEUR (O)): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).unwrap() == 0 { return; }
    let ai_player = if input.trim() == "1" { Player::X } else { Player::O };
    let human_player = ai_player.opponent();
    let mut board = Board::new();
    let params = HeuristicParams::default();
    let mut ctx = SearchContext { tt: HashMap::with_capacity(200000) };

    loop {
        board.print();
        if board.is_terminal() || board.get_available_moves().is_empty() {
            println!("FIN DE PARTIE. Score final: {}", board.evaluate(&params)); break;
        }
        if board.current_player == human_player {
            println!("Tour de l'ADVERSAIRE ({:?})", human_player);
            print!("Entrez Colonne Ligne (ex: 5 5) : "); io::stdout().flush().unwrap();
            let mut m_in = String::new();
            if io::stdin().read_line(&mut m_in).unwrap() == 0 { break; }
            let parts: Vec<&str> = m_in.trim().split_whitespace().collect();
            if parts.len() == 2 {
                if let (Ok(c), Ok(r)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                    if c>=1 && c<=9 && r>=1 && r<=9 {
                        let (cc, rr) = (c-1, r-1);
                        if !board.make_move((rr/3)*3 + (cc/3), (rr%3)*3 + (cc%3)) { println!("Coup INVALIDE !"); }
                        continue;
                    }
                }
            }
            println!("Format invalide ! Tapez 'Colonne Ligne' (1-9).");
        } else {
            println!("Tour de l'IA ({:?}) - Réflexion...", ai_player);
            let start = Instant::now();
            let limit = Duration::from_secs(2);
            let mut best = None;
            for depth in 1..25 {
                let (_, m) = minimax(&board, depth, i32::MIN, i32::MAX, ai_player == Player::X, start, limit, &params, &mut ctx);
                if start.elapsed() >= limit { break; }
                if m.is_some() { best = m; }
                print!("D{} ", depth); io::stdout().flush().unwrap();
            }
            if let Some((m, u)) = best {
                let (c, r) = ((m % 3) * 3 + (u % 3) + 1, (m / 3) * 3 + (u / 3) + 1);
                println!("\n>>> L'IA JOUE : COLONNE {} LIGNE {} <<<", c, r);
                println!("Temps: {:?} | Cache: {}", start.elapsed(), ctx.tt.len());
                board.make_move(m, u);
            }
        }
    }
}
