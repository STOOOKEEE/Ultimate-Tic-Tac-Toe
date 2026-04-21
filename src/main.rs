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
            micro_one: 1,
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
pub enum Player {
    X,
    O,
}

impl Player {
    fn opponent(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellState {
    Empty,
    Player(Player),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MacroState {
    Empty,
    Player(Player),
    Draw,
}

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
                    if self.cells[m][i] == CellState::Empty {
                        moves.push((m, i));
                    }
                }
                return moves;
            }
        }
        
        for m in 0..9 {
            if self.macros[m] == MacroState::Empty {
                for i in 0..9 {
                    if self.cells[m][i] == CellState::Empty {
                        moves.push((m, i));
                    }
                }
            }
        }
        moves
    }

    pub fn make_move(&mut self, macro_idx: usize, micro_idx: usize) -> bool {
        if self.macros[macro_idx] != MacroState::Empty { return false; }
        if self.cells[macro_idx][micro_idx] != CellState::Empty { return false; }
        
        if let Some(active) = self.active_macro {
            if active != macro_idx && self.macros[active] == MacroState::Empty {
                return false;
            }
        }

        self.cells[macro_idx][micro_idx] = CellState::Player(self.current_player);
        self.check_macro_win(macro_idx);
        
        if self.macros[micro_idx] == MacroState::Empty {
            self.active_macro = Some(micro_idx);
        } else {
            self.active_macro = None;
        }

        self.current_player = self.current_player.opponent();
        true
    }

    fn check_macro_win(&mut self, macro_idx: usize) {
        let grid = &self.cells[macro_idx];
        
        for line in WIN_LINES.iter() {
            if grid[line[0]] != CellState::Empty &&
               grid[line[0]] == grid[line[1]] &&
               grid[line[1]] == grid[line[2]] {
                if let CellState::Player(p) = grid[line[0]] {
                    self.macros[macro_idx] = MacroState::Player(p);
                    return;
                }
            }
        }

        if grid.iter().all(|&c| c != CellState::Empty) {
            self.macros[macro_idx] = MacroState::Draw;
        }
    }

    pub fn is_terminal(&self) -> bool {
        for line in WIN_LINES.iter() {
            if self.macros[line[0]] != MacroState::Empty &&
               self.macros[line[0]] != MacroState::Draw &&
               self.macros[line[0]] == self.macros[line[1]] &&
               self.macros[line[1]] == self.macros[line[2]] {
                return true;
            }
        }
        
        self.macros.iter().all(|&m| m != MacroState::Empty)
    }

    pub fn evaluate(&self, params: &HeuristicParams) -> i32 {
        let mut score = 0;

        for line in WIN_LINES.iter() {
            if self.macros[line[0]] != MacroState::Empty &&
               self.macros[line[0]] != MacroState::Draw &&
               self.macros[line[0]] == self.macros[line[1]] &&
               self.macros[line[1]] == self.macros[line[2]] {
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
            let mut x_cnt = 0;
            let mut o_cnt = 0;
            for &idx in line.iter() {
                match grid[idx] {
                    CellState::Player(Player::X) => x_cnt += 1,
                    CellState::Player(Player::O) => o_cnt += 1,
                    CellState::Empty => {}
                }
            }
            if x_cnt > 0 && o_cnt == 0 {
                score += if x_cnt == 2 { params.micro_two } else { params.micro_one };
            } else if o_cnt > 0 && x_cnt == 0 {
                score -= if o_cnt == 2 { params.micro_two } else { params.micro_one };
            }
        }
        if grid[4] == CellState::Player(Player::X) { score += params.micro_center; }
        if grid[4] == CellState::Player(Player::O) { score -= params.micro_center; }
        score
    }

    fn evaluate_macro_potential(&self) -> i32 {
        let mut score = 0;
        for line in WIN_LINES.iter() {
            let mut x_cnt = 0;
            let mut o_cnt = 0;
            for &idx in line.iter() {
                match self.macros[idx] {
                    MacroState::Player(Player::X) => x_cnt += 1,
                    MacroState::Player(Player::O) => o_cnt += 1,
                    _ => {}
                }
            }
            if x_cnt == 2 && o_cnt == 0 { score += 5; }
            if o_cnt == 2 && x_cnt == 0 { score -= 5; }
        }
        score
    }

    pub fn print(&self) {
        println!("  1 2 3   4 5 6   7 8 9 (col)");
        for row in 0..9 {
            if row % 3 == 0 {
                println!("  -----------------------");
            }
            print!("{} ", row + 1);
            for col in 0..9 {
                if col % 3 == 0 {
                    print!("| ");
                }
                let macro_idx = (row / 3) * 3 + (col / 3);
                let micro_idx = (row % 3) * 3 + (col % 3);
                
                if self.macros[macro_idx] == MacroState::Player(Player::X) {
                    print!("X ");
                } else if self.macros[macro_idx] == MacroState::Player(Player::O) {
                    print!("O ");
                } else if self.macros[macro_idx] == MacroState::Draw {
                    print!("- ");
                } else {
                    match self.cells[macro_idx][micro_idx] {
                        CellState::Empty => {
                            if Some(macro_idx) == self.active_macro || self.active_macro.is_none() {
                                print!(". ");
                            } else {
                                print!("  ");
                            }
                        },
                        CellState::Player(Player::X) => print!("x "),
                        CellState::Player(Player::O) => print!("o "),
                    }
                }
            }
            println!("|");
        }
        println!("  -----------------------");
        println!("(Lignes)");
    }
}

pub fn minimax(board: &Board, depth: u32, mut alpha: i32, mut beta: i32, is_maximizing: bool, start_time: Instant, max_time: Duration, params: &HeuristicParams) -> (i32, Option<(usize, usize)>) {
    if depth == 0 || board.is_terminal() || start_time.elapsed() >= max_time {
        return (board.evaluate(params), None);
    }

    let moves = board.get_available_moves();
    if moves.is_empty() {
        return (board.evaluate(params), None);
    }

    let mut best_move = None;

    if is_maximizing {
        let mut max_eval = std::i32::MIN;
        for m in moves {
            let mut new_board = board.clone();
            new_board.make_move(m.0, m.1);
            let (eval, _) = minimax(&new_board, depth - 1, alpha, beta, false, start_time, max_time, params);
            if eval > max_eval {
                max_eval = eval;
                best_move = Some(m);
            }
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        return (max_eval, best_move);
    } else {
        let mut min_eval = std::i32::MAX;
        for m in moves {
            let mut new_board = board.clone();
            new_board.make_move(m.0, m.1);
            let (eval, _) = minimax(&new_board, depth - 1, alpha, beta, true, start_time, max_time, params);
            if eval < min_eval {
                min_eval = eval;
                best_move = Some(m);
            }
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        return (min_eval, best_move);
    }
}

fn parse_input(input: &str) -> Option<(usize, usize)> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.len() != 2 { return None; }
    
    let col: usize = parts[0].parse().ok()?;
    let row: usize = parts[1].parse().ok()?;
    
    if col < 1 || col > 9 || row < 1 || row > 9 { return None; }
    
    let c = col - 1;
    let r = row - 1;
    let macro_idx = (r / 3) * 3 + (c / 3);
    let micro_idx = (r % 3) * 3 + (c % 3);
    
    Some((macro_idx, micro_idx))
}

fn play_game(params_x: &HeuristicParams, params_o: &HeuristicParams, display: bool) -> i32 {
    let mut board = Board::new();
    let max_depth = 4;
    let time_limit = Duration::from_millis(50);
    
    let mut r = rng();
    let first_moves = board.get_available_moves();
    let first_m = first_moves[r.random_range(0..first_moves.len())];
    board.make_move(first_m.0, first_m.1);

    loop {
        if board.is_terminal() || board.get_available_moves().is_empty() {
            let eval = board.evaluate(&HeuristicParams::default());
            if display { board.print(); }
            if eval > 500000 { return 1; }
            if eval < -500000 { return -1; }
            return 0;
        }
        
        let start = Instant::now();
        let is_max = board.current_player == Player::X;
        let current_params = if is_max { params_x } else { params_o };
        let (_, best_move) = minimax(&board, max_depth, std::i32::MIN, std::i32::MAX, is_max, start, time_limit, current_params);
        
        if let Some((m_idx, u_idx)) = best_move {
            board.make_move(m_idx, u_idx);
        } else {
            let moves = board.get_available_moves();
            if moves.is_empty() { break; }
            board.make_move(moves[0].0, moves[0].1);
        }
    }
    
    let eval = board.evaluate(&HeuristicParams::default());
    if eval > 500000 { 1 } else if eval < -500000 { -1 } else { 0 }
}

fn run_train() {
    println!("--- Lancement de l'Entraînement (Recherche Heuristique) ---");
    let mut best_params = HeuristicParams::default();
    let generations = 5; 
    let matches_per_gen = 6;
    
    for generation_idx in 1..=generations {
        println!("Génération {}/{} en cours...", generation_idx, generations);
        let candidate = best_params.mutate();
        
        let mut score_best = 0;
        let mut score_candidate = 0;
        
        for i in 0..matches_per_gen {
            let (winner, p_x, p_o) = if i % 2 == 0 {
                (play_game(&best_params, &candidate, false), "Best", "Cand")
            } else {
                (play_game(&candidate, &best_params, false), "Cand", "Best")
            };
            
            if (winner == 1 && p_x == "Best") || (winner == -1 && p_o == "Best") {
                score_best += 1;
            } else if (winner == 1 && p_x == "Cand") || (winner == -1 && p_o == "Cand") {
                score_candidate += 1;
            }
        }
        
        println!("  Résultat: Actuel {} - {} Candidat", score_best, score_candidate);
        if score_candidate > score_best {
            println!("  -> ✨ NOUVELLE MEILLEURE HEURISTIQUE ! ✨");
            best_params = candidate;
            println!("     {:?}", best_params);
        }
    }
    
    println!("--- Entraînement terminé ---");
    println!("Meilleurs paramètres trouvés :");
    println!("{:#?}", best_params);
}

fn run_arena() {
    println!("--- Lancement de l'Arène IA vs IA ---");
    let mut x_wins = 0;
    let mut o_wins = 0;
    let mut draws = 0;
    let games = 10;
    let params = HeuristicParams::default();

    for i in 1..=games {
        println!("Partie {} en cours...", i);
        let winner = play_game(&params, &params, false);
        if winner == 1 {
            x_wins += 1;
            println!(" -> X Gagne");
        } else if winner == -1 {
            o_wins += 1;
            println!(" -> O Gagne");
        } else {
            draws += 1;
            println!(" -> Égalité");
        }
    }
    
    println!("--- Résultats ---");
    println!("Victoires X : {}", x_wins);
    println!("Victoires O : {}", o_wins);
    println!("Égalités    : {}", draws);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        if args[1] == "arena" {
            run_arena();
            return;
        } else if args[1] == "train" {
            run_train();
            return;
        }
    }

    println!("--- Ultimate Tic Tac Toe ---");
    println!("1. Jouer en premier (X)");
    println!("2. Laisser l'IA commencer (O)");
    print!("Choix (1/2): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    let bytes = io::stdin().read_line(&mut input).unwrap();
    if bytes == 0 { return; }
    
    let human_player = if input.trim() == "2" { Player::O } else { Player::X };
    let ai_player = human_player.opponent();
    let params = HeuristicParams::default();
    
    let mut board = Board::new();
    
    let max_depth = 6;
    let time_limit = Duration::from_secs(2);
    
    loop {
        board.print();
        
        if board.is_terminal() || board.get_available_moves().is_empty() {
            println!("Partie terminée!");
            let eval = board.evaluate(&params);
            if eval > 500000 {
                println!("Joueur X gagne !");
            } else if eval < -500000 {
                println!("Joueur O gagne !");
            } else {
                println!("Match nul !");
            }
            break;
        }
        
        if board.current_player == human_player {
            println!("Votre tour ({:?}). Entrez colonne (1-9) puis ligne (1-9) séparés par un espace:", human_player);
            print!("> ");
            io::stdout().flush().unwrap();
            
            let mut move_input = String::new();
            let bytes = io::stdin().read_line(&mut move_input).unwrap();
            if bytes == 0 {
                println!("Fin de partie (entrée fermée).");
                break;
            }
            
            if let Some((m_idx, u_idx)) = parse_input(&move_input) {
                if !board.make_move(m_idx, u_idx) {
                    println!("Coup invalide. Réessayez.");
                }
            } else {
                println!("Format invalide. Exemple: '5 5' pour colonne 5, ligne 5.");
            }
        } else {
            println!("Tour de l'IA ({:?}). Réflexion en cours...", ai_player);
            let start = Instant::now();
            let is_max = ai_player == Player::X;
            let (_, best_move) = minimax(&board, max_depth, std::i32::MIN, std::i32::MAX, is_max, start, time_limit, &params);
            
            if let Some((m_idx, u_idx)) = best_move {
                let c = (m_idx % 3) * 3 + (u_idx % 3);
                let r = (m_idx / 3) * 3 + (u_idx / 3);
                println!("L'IA joue Colonne {} Ligne {}", c + 1, r + 1);
                board.make_move(m_idx, u_idx);
            } else {
                println!("Erreur: L'IA n'a pas trouvé de coup !");
                break;
            }
            let duration = start.elapsed();
            println!("Temps de réflexion: {:?}", duration);
        }
    }
}
