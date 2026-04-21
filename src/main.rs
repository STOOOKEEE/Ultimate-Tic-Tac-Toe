use std::io::{self, Write};
use std::time::{Instant, Duration};
use rand::{rng, RngExt};
use std::env;

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
            center_macro_mult: 1.5,
            micro_two: 10,
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
            0 => new.macro_win += r.random_range(-200..=200),
            1 => new.center_macro_mult += r.random_range(-0.5..=0.5),
            2 => new.micro_two += r.random_range(-5..=5),
            3 => new.micro_one += r.random_range(-2..=2),
            4 => new.micro_center += r.random_range(-2..=2),
            5 => new.macro_two += r.random_range(-50..=50),
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
}

const WIN_LINES: [[usize; 3]; 8] = [
    [0, 1, 2], [3, 4, 5], [6, 7, 8],
    [0, 3, 6], [1, 4, 7], [2, 5, 8],
    [0, 4, 8], [2, 4, 6]
];

// Move priority: Center > Corners > Edges
const MOVE_PRIORITY: [i32; 9] = [2, 1, 2, 1, 3, 1, 2, 1, 2];

impl Board {
    pub fn new() -> Self {
        Self {
            cells: [[CellState::Empty; 9]; 9],
            macros: [MacroState::Empty; 9],
            active_macro: None,
            current_player: Player::X,
        }
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
        self.cells[macro_idx][micro_idx] = CellState::Player(self.current_player);
        self.check_macro_win(macro_idx);
        self.active_macro = if self.macros[micro_idx] == MacroState::Empty { Some(micro_idx) } else { None };
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
        
        // Global Win Check
        for line in WIN_LINES.iter() {
            if self.macros[line[0]] != MacroState::Empty && self.macros[line[0]] != MacroState::Draw &&
               self.macros[line[0]] == self.macros[line[1]] && self.macros[line[1]] == self.macros[line[2]] {
                if let MacroState::Player(p) = self.macros[line[0]] {
                    return if p == Player::X { 1000000 } else { -1000000 };
                }
            }
        }

        // Active Macro Penalty: If we send the opponent to a board where they are close to winning
        if let Some(target_macro) = self.active_macro {
            let grid = &self.cells[target_macro];
            for line in WIN_LINES.iter() {
                let (mut x, mut o) = (0, 0);
                for &idx in line.iter() {
                    match grid[idx] {
                        CellState::Player(Player::X) => x += 1,
                        CellState::Player(Player::O) => o += 1,
                        CellState::Empty => {}
                    }
                }
                // If it's O's turn (current_player), and we just sent them to a board where X has 2
                if self.current_player == Player::O && x == 2 && o == 0 {
                    score += 150; // Bonus for X (malus for O who was sent there)
                } else if self.current_player == Player::X && o == 2 && x == 0 {
                    score -= 150; // Bonus for O
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

pub fn minimax(board: &Board, depth: u32, mut alpha: i32, mut beta: i32, is_maximizing: bool, start_time: Instant, max_time: Duration, params: &HeuristicParams) -> (i32, Option<(usize, usize)>) {
    if depth == 0 || board.is_terminal() || start_time.elapsed() >= max_time {
        return (board.evaluate(params), None);
    }

    let mut moves = board.get_available_moves();
    if moves.is_empty() { return (board.evaluate(params), None); }

    // Move Ordering: Priority based on MOVE_PRIORITY
    moves.sort_by_key(|&(_, u_idx)| std::cmp::Reverse(MOVE_PRIORITY[u_idx]));

    let mut best_move = None;
    if is_maximizing {
        let mut max_eval = i32::MIN;
        for m in moves {
            let mut new_board = board.clone();
            new_board.make_move(m.0, m.1);
            let (eval, _) = minimax(&new_board, depth - 1, alpha, beta, false, start_time, max_time, params);
            // Add tiny random noise to break ties
            let noise = if depth > 1 { (m.1 % 3) as i32 } else { 0 };
            let score = eval + noise;
            
            if score > max_eval { max_eval = score; best_move = Some(m); }
            alpha = alpha.max(score);
            if beta <= alpha { break; }
            if start_time.elapsed() >= max_time { break; }
        }
        (max_eval, best_move)
    } else {
        let mut min_eval = i32::MAX;
        for m in moves {
            let mut new_board = board.clone();
            new_board.make_move(m.0, m.1);
            let (eval, _) = minimax(&new_board, depth - 1, alpha, beta, true, start_time, max_time, params);
            let noise = if depth > 1 { (m.1 % 3) as i32 } else { 0 };
            let score = eval - noise;

            if score < min_eval { min_eval = score; best_move = Some(m); }
            beta = beta.min(score);
            if beta <= alpha { break; }
            if start_time.elapsed() >= max_time { break; }
        }
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

fn play_game(params_x: &HeuristicParams, params_o: &HeuristicParams) -> i32 {
    let mut board = Board::new();
    let time_limit = Duration::from_millis(100);
    let mut r = rng();
    let moves = board.get_available_moves();
    let first = moves[r.random_range(0..moves.len())];
    board.make_move(first.0, first.1);

    loop {
        if board.is_terminal() || board.get_available_moves().is_empty() {
            let eval = board.evaluate(&HeuristicParams::default());
            return if eval > 500000 { 1 } else if eval < -500000 { -1 } else { 0 };
        }
        let start = Instant::now();
        let is_max = board.current_player == Player::X;
        let p = if is_max { params_x } else { params_o };
        // Fixed depth for simulation/training
        let (_, best_move) = minimax(&board, 4, i32::MIN, i32::MAX, is_max, start, time_limit, p);
        if let Some(m) = best_move { board.make_move(m.0, m.1); }
        else { break; }
    }
    0
}

fn run_train() {
    println!("--- Lancement de l'Entraînement ---");
    let mut best_params = HeuristicParams::default();
    for generation_idx in 1..=5 {
        println!("Génération {}/5...", generation_idx);
        let cand = best_params.mutate();
        let (mut b, mut c) = (0, 0);
        for i in 0..6 {
            let res = if i % 2 == 0 { play_game(&best_params, &cand) } else { -play_game(&cand, &best_params) };
            if res > 0 { b += 1; } else if res < 0 { c += 1; }
        }
        if c > b { println!("-> Nouveau record !"); best_params = cand; }
    }
    println!("Meilleurs paramètres : {:#?}", best_params);
}

fn run_arena() {
    let mut x_wins = 0;
    let mut o_wins = 0;
    let p = HeuristicParams::default();
    for i in 1..=10 {
        println!("Match {}...", i);
        let res = play_game(&p, &p);
        if res == 1 { x_wins += 1; } else if res == -1 { o_wins += 1; }
    }
    println!("X: {}, O: {}", x_wins, o_wins);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args[1] == "arena" { run_arena(); return; }
        if args[1] == "train" { run_train(); return; }
    }
    println!("--- Ultimate Tic Tac Toe PRO ---");
    print!("Qui commence ? (1: Joueur X, 2: IA O): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).unwrap() == 0 { return; }
    let human = if input.trim() == "2" { Player::O } else { Player::X };
    let ai = human.opponent();
    let params = HeuristicParams::default();
    let mut board = Board::new();

    loop {
        board.print();
        if board.is_terminal() || board.get_available_moves().is_empty() {
            println!("Terminé ! Resultat: {}", board.evaluate(&params)); break;
        }
        if board.current_player == human {
            print!("Votre tour (Col Ligne): "); io::stdout().flush().unwrap();
            let mut m_in = String::new();
            if io::stdin().read_line(&mut m_in).unwrap() == 0 { break; }
            if let Some((m, u)) = parse_input(&m_in) {
                if !board.make_move(m, u) { println!("Invalide !"); }
            }
        } else {
            println!("IA réfléchit...");
            let start = Instant::now();
            let time_limit = Duration::from_secs(2);
            let mut best_so_far = None;
            // ITERATIVE DEEPENING
            for depth in 1..20 {
                let (_, m) = minimax(&board, depth, i32::MIN, i32::MAX, ai == Player::X, start, time_limit, &params);
                if start.elapsed() >= time_limit { break; }
                if m.is_some() { best_so_far = m; }
                println!("  Profondeur {} complétée...", depth);
            }
            if let Some((m, u)) = best_so_far {
                println!("IA joue: Col {} Ligne {}", (m % 3) * 3 + (u % 3) + 1, (m / 3) * 3 + (u / 3) + 1);
                board.make_move(m, u);
            }
        }
    }
}
