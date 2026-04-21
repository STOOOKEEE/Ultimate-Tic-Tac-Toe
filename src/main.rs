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
            macro_win: 1000,
            center_macro_mult: 1.2248424,
            micro_two: 15,
            micro_one: 0,
            micro_center: 0,
            macro_two: 100,
        }
    }
}

impl HeuristicParams {
    pub fn mutate(&self) -> Self {
        let mut r = rng();
        let mut new = *self;
        match r.random_range(0..6) {
            0 => new.macro_win += r.random_range(-100..=100),
            1 => new.center_macro_mult += r.random_range(-0.2..=0.2),
            2 => new.micro_two += r.random_range(-3..=3),
            3 => new.micro_one += r.random_range(-1..=1),
            4 => new.micro_center += r.random_range(-1..=1),
            5 => new.macro_two += r.random_range(-20..=20),
            _ => {}
        }
        if new.macro_win < 100 { new.macro_win = 100; }
        if new.center_macro_mult < 1.0 { new.center_macro_mult = 1.0; }
        if new.micro_two < 1 { new.micro_two = 1; }
        if new.micro_one < 0 { new.micro_one = 0; }
        if new.micro_center < 0 { new.micro_center = 0; }
        if new.macro_two < 1 { new.macro_two = 1; }
        new
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

// Zobrist Hashing Keys
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
        
        self.hash ^= ZOBRIST.active_macro[old_active];
        self.hash ^= ZOBRIST.active_macro[new_active];
        self.hash ^= ZOBRIST.player;
        
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
            let mut x_macros = 0;
            let mut o_macros = 0;
            for m in self.macros.iter() {
                if *m == MacroState::Player(Player::X) { x_macros += 1; }
                if *m == MacroState::Player(Player::O) { o_macros += 1; }
            }
            if x_macros > o_macros { return 1000000; }
            if o_macros > x_macros { return -1000000; }
            return 0;
        }

        if let Some(target_macro) = self.active_macro {
            let grid = &self.cells[target_macro];
            for line in WIN_LINES.iter() {
                let (mut x, mut o) = (0, 0);
                for &idx in line.iter() {
                    match grid[idx] {
                        CellState::Player(Player::X) => x += 1,
                        CellState::Player(Player::O) => o += 1,
                        _ => {}
                    }
                }
                if self.current_player == Player::O && x == 2 && o == 0 { score += 150; }
                else if self.current_player == Player::X && o == 2 && x == 0 { score -= 150; }
            }
        }

        for m in 0..9 {
            let multiplier = if m == 4 { params.center_macro_mult } else { 1.0 };
            match self.macros[m] {
                MacroState::Player(Player::X) => score += (params.macro_win as f32 * multiplier) as i32,
                MacroState::Player(Player::O) => score -= (params.macro_win as f32 * multiplier) as i32,
                _ => {
                    let micro_score = self.evaluate_grid(&self.cells[m], params);
                    score += (micro_score as f32 * multiplier) as i32;
                }
            }
        }
        score += self.evaluate_macro_potential() * params.macro_two;
        score
    }

    fn evaluate_grid(&self, grid: &[CellState; 9], params: &HeuristicParams) -> i32 {
        let mut score = 0;
        for line in WIN_LINES.iter() {
            let (mut x, mut o) = (0, 0);
            for &idx in line.iter() {
                match grid[idx] {
                    CellState::Player(Player::X) => x += 1,
                    CellState::Player(Player::O) => o += 1,
                    CellState::Empty => {}
                }
            }
            if x > 0 && o == 0 { score += if x == 2 { params.micro_two } else { params.micro_one }; }
            else if o > 0 && x == 0 { score -= if o == 2 { params.micro_two } else { params.micro_one }; }
        }
        if grid[4] == CellState::Player(Player::X) { score += params.micro_center; }
        if grid[4] == CellState::Player(Player::O) { score -= params.micro_center; }
        score
    }

    fn evaluate_macro_potential(&self) -> i32 {
        let mut score = 0;
        for line in WIN_LINES.iter() {
            let (mut x, mut o) = (0, 0);
            for &idx in line.iter() {
                match self.macros[idx] {
                    MacroState::Player(Player::X) => x += 1,
                    MacroState::Player(Player::O) => o += 1,
                    _ => {}
                }
            }
            if x == 2 && o == 0 { score += 5; }
            if o == 2 && x == 0 { score -= 5; }
        }
        score
    }

    pub fn print(&self) {
        println!("  1 2 3   4 5 6   7 8 9 (col)");
        for row in 0..9 {
            if row % 3 == 0 { println!("  -----------------------"); }
            print!("{} ", row + 1);
            for col in 0..9 {
                if col % 3 == 0 { print!("| "); }
                let (m_idx, u_idx) = ((row / 3) * 3 + (col / 3), (row % 3) * 3 + (col % 3));
                if self.macros[m_idx] == MacroState::Player(Player::X) { print!("X "); }
                else if self.macros[m_idx] == MacroState::Player(Player::O) { print!("O "); }
                else if self.macros[m_idx] == MacroState::Draw { print!("- "); }
                else {
                    match self.cells[m_idx][u_idx] {
                        CellState::Empty => {
                            if Some(m_idx) == self.active_macro || self.active_macro.is_none() { print!(". "); }
                            else { print!("  "); }
                        },
                        CellState::Player(Player::X) => print!("x "),
                        CellState::Player(Player::O) => print!("o "),
                    }
                }
            }
            println!("|");
        }
        println!("  -----------------------");
    }
}

#[derive(Clone, Copy)]
struct TransEntry {
    depth: u32,
    score: i32,
}

pub struct SearchContext {
    pub tt: HashMap<u64, TransEntry>,
}

pub fn minimax(board: &Board, depth: u32, mut alpha: i32, mut beta: i32, is_maximizing: bool, start_time: Instant, max_time: Duration, params: &HeuristicParams, ctx: &mut SearchContext) -> (i32, Option<(usize, usize)>) {
    if let Some(entry) = ctx.tt.get(&board.hash) {
        if entry.depth >= depth { return (entry.score, None); }
    }
    
    if depth == 0 || board.is_terminal() || start_time.elapsed() >= max_time {
        let score = board.evaluate(params);
        return (score, None);
    }

    let mut moves = board.get_available_moves();
    if moves.is_empty() { return (board.evaluate(params), None); }

    moves.sort_by_key(|&(_, u_idx)| std::cmp::Reverse(MOVE_PRIORITY[u_idx]));

    let mut best_move = None;
    if is_maximizing {
        let mut max_eval = i32::MIN;
        for m in moves {
            let mut new_board = board.clone();
            new_board.make_move(m.0, m.1);
            let (eval, _) = minimax(&new_board, depth - 1, alpha, beta, false, start_time, max_time, params, ctx);
            let score = eval + if depth > 1 { (m.1 % 3) as i32 } else { 0 };
            if score > max_eval { max_eval = score; best_move = Some(m); }
            alpha = alpha.max(score);
            if beta <= alpha || start_time.elapsed() >= max_time { break; }
        }
        ctx.tt.insert(board.hash, TransEntry { depth, score: max_eval });
        (max_eval, best_move)
    } else {
        let mut min_eval = i32::MAX;
        for m in moves {
            let mut new_board = board.clone();
            new_board.make_move(m.0, m.1);
            let (eval, _) = minimax(&new_board, depth - 1, alpha, beta, true, start_time, max_time, params, ctx);
            let score = eval - if depth > 1 { (m.1 % 3) as i32 } else { 0 };
            if score < min_eval { min_eval = score; best_move = Some(m); }
            beta = beta.min(score);
            if beta <= alpha || start_time.elapsed() >= max_time { break; }
        }
        ctx.tt.insert(board.hash, TransEntry { depth, score: min_eval });
        (min_eval, best_move)
    }
}

fn parse_input(input: &str) -> Option<(usize, usize)> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.len() != 2 { return None; }
    let (col, row): (usize, usize) = (parts[0].parse().ok()?, parts[1].parse().ok()?);
    if col < 1 || col > 9 || row < 1 || row > 9 { return None; }
    let (c, r) = (col - 1, row - 1);
    Some(((r / 3) * 3 + (c / 3), (r % 3) * 3 + (c % 3)))
}

fn play_game(params_x: &HeuristicParams, params_o: &HeuristicParams, depth: u32) -> i32 {
    let mut board = Board::new();
    let mut ctx = SearchContext { tt: HashMap::with_capacity(10000) };
    let mut r = rng();
    let moves = board.get_available_moves();
    let first = moves[r.random_range(0..moves.len())];
    board.make_move(first.0, first.1);

    loop {
        if board.is_terminal() || board.get_available_moves().is_empty() {
            let eval = board.evaluate(&HeuristicParams::default());
            return if eval > 500000 { 1 } else if eval < -500000 { -1 } else { 0 };
        }
        let is_max = board.current_player == Player::X;
        let p = if is_max { params_x } else { params_o };
        let (_, best_move) = minimax(&board, depth, i32::MIN, i32::MAX, is_max, Instant::now(), Duration::from_secs(10), p, &mut ctx);
        if let Some(m) = best_move { board.make_move(m.0, m.1); }
        else { break; }
    }
    0
}

fn run_train() {
    println!("--- Lancement de l'Entraînement LOURD ---");
    let mut best_params = HeuristicParams::default();
    let generations = 10;
    let matches_per_gen = 6;
    let search_depth = 15; // Extreme depth for ultra-robust heuristic

    for generation_idx in 1..=generations {
        print!("Génération {}/{} (D{})... ", generation_idx, generations, search_depth);
        io::stdout().flush().unwrap();
        let cand = best_params.mutate();
        let (mut b, mut c, mut d) = (0, 0, 0);
        for i in 0..matches_per_gen {
            let res = if i % 2 == 0 { play_game(&best_params, &cand, search_depth) } else { -play_game(&cand, &best_params, search_depth) };
            if res > 0 { b += 1; } else if res < 0 { c += 1; } else { d += 1; }
        }
        println!("Best {} - {} Cand ({} Nuls)", b, c, d);
        if c > b { println!("  -> ✨ NOUVEAU RECORD !"); best_params = cand; }
    }
    println!("Meilleurs paramètres : {:#?}", best_params);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args[1] == "arena" { 
            let p = HeuristicParams::default();
            for i in 1..=5 { println!("Match {}... X:{}", i, play_game(&p, &p, 6)); }
            return;
        }
        if args[1] == "train" { run_train(); return; }
    }
    println!("--- Ultimate Tic Tac Toe ULTRA ---");
    print!("1: Joueur X, 2: IA O: "); io::stdout().flush().unwrap();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).unwrap() == 0 { return; }
    let human = if input.trim() == "2" { Player::O } else { Player::X };
    let ai = human.opponent();
    let params = HeuristicParams::default();
    let mut board = Board::new();
    let mut ctx = SearchContext { tt: HashMap::with_capacity(100000) };

    loop {
        board.print();
        if board.is_terminal() || board.get_available_moves().is_empty() { break; }
        if board.current_player == human {
            print!("Coup (Col Lig): "); io::stdout().flush().unwrap();
            let mut m_in = String::new();
            if io::stdin().read_line(&mut m_in).unwrap() == 0 { break; }
            if let Some((m, u)) = parse_input(&m_in) { board.make_move(m, u); }
        } else {
            println!("IA réfléchit (TT size: {})...", ctx.tt.len());
            let start = Instant::now();
            let limit = Duration::from_secs(2);
            let mut best = None;
            for depth in 1..25 {
                let (_, m) = minimax(&board, depth, i32::MIN, i32::MAX, ai == Player::X, start, limit, &params, &mut ctx);
                if start.elapsed() >= limit { break; }
                if m.is_some() { best = m; }
                print!("D{} ", depth); io::stdout().flush().unwrap();
            }
            if let Some((m, u)) = best {
                println!("\nIA joue: {} {}", (m % 3) * 3 + (u % 3) + 1, (m / 3) * 3 + (u / 3) + 1);
                board.make_move(m, u);
            }
        }
    }
}
