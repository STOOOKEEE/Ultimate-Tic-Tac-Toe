use crate::ai::{MAX_SEARCH_DEPTH, SearchReport, find_best_move};
use crate::coords::{coordinates_to_move, move_to_coordinates};
use crate::game::{Board, GameOutcome, HeuristicParams, Player};
use std::io::{self, Write};
use std::time::Duration;

enum GameMode {
    HumanVsAi,
    HumanVsHuman,
    AiVsAi,
    Benchmark,
    Tournament,
}

struct AiConfig {
    name: String,
    time_limit: Duration,
    max_depth: u32,
}

struct GameStats {
    outcome: GameOutcome,
    moves: usize,
    x_boards: usize,
    o_boards: usize,
    x_time: Duration,
    o_time: Duration,
    x_moves: usize,
    o_moves: usize,
    x_depth_total: u64,
    o_depth_total: u64,
}

impl GameStats {
    fn new() -> Self {
        Self {
            outcome: GameOutcome::Ongoing,
            moves: 0,
            x_boards: 0,
            o_boards: 0,
            x_time: Duration::ZERO,
            o_time: Duration::ZERO,
            x_moves: 0,
            o_moves: 0,
            x_depth_total: 0,
            o_depth_total: 0,
        }
    }

    fn record_search(&mut self, player: Player, report: &SearchReport) {
        self.moves += 1;

        match player {
            Player::X => {
                self.x_moves += 1;
                self.x_time += report.elapsed;
                self.x_depth_total += u64::from(report.completed_depth);
            }
            Player::O => {
                self.o_moves += 1;
                self.o_time += report.elapsed;
                self.o_depth_total += u64::from(report.completed_depth);
            }
        }
    }

    fn moves_for(&self, player: Player) -> usize {
        match player {
            Player::X => self.x_moves,
            Player::O => self.o_moves,
        }
    }

    fn time_for(&self, player: Player) -> Duration {
        match player {
            Player::X => self.x_time,
            Player::O => self.o_time,
        }
    }

    fn depth_total_for(&self, player: Player) -> u64 {
        match player {
            Player::X => self.x_depth_total,
            Player::O => self.o_depth_total,
        }
    }
}

struct BenchmarkStats {
    main_wins: usize,
    opponent_wins: usize,
    draws: usize,
    x_wins: usize,
    o_wins: usize,
    total_moves: usize,
    main_moves: usize,
    opponent_moves: usize,
    main_time: Duration,
    opponent_time: Duration,
    main_depth_total: u64,
    opponent_depth_total: u64,
    main_as_x_wins: usize,
    main_as_x_losses: usize,
    main_as_x_draws: usize,
    main_as_o_wins: usize,
    main_as_o_losses: usize,
    main_as_o_draws: usize,
}

impl BenchmarkStats {
    fn new() -> Self {
        Self {
            main_wins: 0,
            opponent_wins: 0,
            draws: 0,
            x_wins: 0,
            o_wins: 0,
            total_moves: 0,
            main_moves: 0,
            opponent_moves: 0,
            main_time: Duration::ZERO,
            opponent_time: Duration::ZERO,
            main_depth_total: 0,
            opponent_depth_total: 0,
            main_as_x_wins: 0,
            main_as_x_losses: 0,
            main_as_x_draws: 0,
            main_as_o_wins: 0,
            main_as_o_losses: 0,
            main_as_o_draws: 0,
        }
    }

    fn record_game(&mut self, stats: &GameStats, main_player: Player) {
        self.total_moves += stats.moves;

        match (main_player, outcome_winner(stats.outcome)) {
            (Player::X, Some(winner)) if winner == main_player => {
                self.main_wins += 1;
                self.main_as_x_wins += 1;
            }
            (Player::X, Some(_)) => {
                self.opponent_wins += 1;
                self.main_as_x_losses += 1;
            }
            (Player::X, None) => {
                self.draws += 1;
                self.main_as_x_draws += 1;
            }
            (Player::O, Some(winner)) if winner == main_player => {
                self.main_wins += 1;
                self.main_as_o_wins += 1;
            }
            (Player::O, Some(_)) => {
                self.opponent_wins += 1;
                self.main_as_o_losses += 1;
            }
            (Player::O, None) => {
                self.draws += 1;
                self.main_as_o_draws += 1;
            }
        }

        match outcome_winner(stats.outcome) {
            Some(Player::X) => self.x_wins += 1,
            Some(Player::O) => self.o_wins += 1,
            None => {}
        }

        let opponent = main_player.opponent();
        self.main_moves += stats.moves_for(main_player);
        self.opponent_moves += stats.moves_for(opponent);
        self.main_time += stats.time_for(main_player);
        self.opponent_time += stats.time_for(opponent);
        self.main_depth_total += stats.depth_total_for(main_player);
        self.opponent_depth_total += stats.depth_total_for(opponent);
    }
}

pub(crate) fn run() -> io::Result<()> {
    println!("=== ULTIMATE TIC TAC TOE ===");
    println!("Saisie des coups: colonne puis ligne, valeurs de 1 a 9.");

    match read_game_mode()? {
        GameMode::HumanVsAi => run_human_vs_ai(),
        GameMode::HumanVsHuman => run_human_vs_human(),
        GameMode::AiVsAi => run_ai_vs_ai(),
        GameMode::Benchmark => run_benchmark(),
        GameMode::Tournament => run_tournament(),
    }
}

fn run_human_vs_ai() -> io::Result<()> {
    let ai_player = read_ai_player()?;
    let human_player = ai_player.opponent();
    let mut board = Board::new();
    let params = HeuristicParams::default();

    println!("IA: {ai_player} | Joueur: {human_player}");

    loop {
        board.print();

        if board.is_terminal() || board.get_available_moves().is_empty() {
            print_game_result(&board);
            break;
        }

        if board.current_player() == human_player {
            println!("Tour du joueur ({human_player})");
            if !read_human_move(&mut board)? {
                break;
            }
        } else {
            println!("Tour de l'IA ({ai_player}) - Reflexion...");
            let report = find_best_move(&board, &params, Duration::from_secs(2), MAX_SEARCH_DEPTH);

            if let Some(best_move) = report.best_move {
                let (column, row) = move_to_coordinates(best_move);
                println!("L'IA joue: colonne {column} ligne {row}");
                println!(
                    "Temps: {:?} | profondeur complete: {} | cache: {}",
                    report.elapsed, report.completed_depth, report.cache_size
                );
                board.make_move(best_move.0, best_move.1);
            } else {
                println!("Aucun coup legal disponible.");
                print_game_result(&board);
                break;
            }
        }
    }

    Ok(())
}

fn run_human_vs_human() -> io::Result<()> {
    let mut board = Board::new();

    println!("Mode joueur contre joueur. X commence.");

    loop {
        board.print();

        if board.is_terminal() || board.get_available_moves().is_empty() {
            print_game_result(&board);
            break;
        }

        println!("Tour du joueur ({})", board.current_player());
        if !read_human_move(&mut board)? {
            break;
        }
    }

    Ok(())
}

fn run_ai_vs_ai() -> io::Result<()> {
    let params = HeuristicParams::default();
    let x_ai = AiConfig {
        name: String::from("IA X"),
        time_limit: Duration::from_secs(2),
        max_depth: MAX_SEARCH_DEPTH,
    };
    let o_ai = AiConfig {
        name: String::from("IA O"),
        time_limit: Duration::from_secs(2),
        max_depth: MAX_SEARCH_DEPTH,
    };

    println!("Mode IA contre IA. Budget: 2 secondes par coup.");
    let stats = play_ai_game(&x_ai, &o_ai, &params, true);
    print_ai_game_stats(&stats);

    Ok(())
}

fn run_benchmark() -> io::Result<()> {
    let games = read_usize_with_default("Nombre de parties benchmark", 10)?;
    let time_ms = read_u64_with_default("Budget par coup en ms", 100)?;
    let opponent_depth = read_usize_with_default("Profondeur de l'IA faible adverse", 1)?
        .min(MAX_SEARCH_DEPTH as usize) as u32;
    let params = HeuristicParams::default();
    let main_ai = AiConfig {
        name: String::from("IA principale"),
        time_limit: Duration::from_millis(time_ms.max(1)),
        max_depth: MAX_SEARCH_DEPTH,
    };
    let opponent_ai = AiConfig {
        name: format!("IA faible P{opponent_depth}"),
        time_limit: Duration::from_millis(time_ms.max(1)),
        max_depth: opponent_depth,
    };
    let mut benchmark = BenchmarkStats::new();

    println!(
        "Benchmark: {games} parties, budget {} ms/coup, adversaire profondeur {opponent_depth}.",
        time_ms.max(1)
    );
    println!("L'IA principale alterne entre X et O.");

    for game_idx in 0..games {
        let main_player = if game_idx % 2 == 0 {
            Player::X
        } else {
            Player::O
        };
        let (x_ai, o_ai) = if main_player == Player::X {
            (&main_ai, &opponent_ai)
        } else {
            (&opponent_ai, &main_ai)
        };

        let stats = play_ai_game(x_ai, o_ai, &params, false);
        benchmark.record_game(&stats, main_player);
        print_benchmark_game(game_idx + 1, main_player, &stats);
    }

    print_benchmark_summary(games, &benchmark);

    Ok(())
}

fn run_tournament() -> io::Result<()> {
    let ai_player = read_ai_player()?;
    let human_player = ai_player.opponent();
    let time_ms = read_u64_with_default("Budget IA tournoi en ms", 2000)?;
    let mut board = Board::new();
    let params = HeuristicParams::default();

    println!("Mode tournoi: l'IA imprime seulement `colonne ligne`.");

    loop {
        if board.is_terminal() || board.get_available_moves().is_empty() {
            print_compact_game_result(&board);
            break;
        }

        if board.current_player() == ai_player {
            let report = find_best_move(
                &board,
                &params,
                Duration::from_millis(time_ms.max(1)),
                MAX_SEARCH_DEPTH,
            );

            if let Some(best_move) = report.best_move {
                let (column, row) = move_to_coordinates(best_move);
                println!("{column} {row}");
                board.make_move(best_move.0, best_move.1);
            } else {
                print_compact_game_result(&board);
                break;
            }
        } else {
            eprint!("adversaire {human_player}> ");
            io::stderr().flush()?;
            if !read_compact_human_move(&mut board)? {
                break;
            }
        }
    }

    Ok(())
}

fn play_ai_game(
    x_ai: &AiConfig,
    o_ai: &AiConfig,
    params: &HeuristicParams,
    verbose: bool,
) -> GameStats {
    let mut board = Board::new();
    let mut stats = GameStats::new();

    loop {
        if verbose {
            board.print();
        }

        if board.is_terminal() || board.get_available_moves().is_empty() {
            stats.outcome = board.outcome();
            (stats.x_boards, stats.o_boards) = board.local_board_counts();
            if verbose {
                print_game_result(&board);
            }
            return stats;
        }

        let player = board.current_player();
        let ai = if player == Player::X { x_ai } else { o_ai };

        if verbose {
            println!("Tour de {} ({player}) - Reflexion...", ai.name);
        }

        let report = find_best_move(&board, params, ai.time_limit, ai.max_depth);
        let Some(best_move) = report.best_move else {
            stats.outcome = board.outcome();
            (stats.x_boards, stats.o_boards) = board.local_board_counts();
            if verbose {
                println!("Aucun coup legal disponible.");
                print_game_result(&board);
            }
            return stats;
        };

        let (column, row) = move_to_coordinates(best_move);

        if verbose {
            println!("{} joue: colonne {column} ligne {row}", ai.name);
            println!(
                "Temps: {:?} | profondeur complete: {} | cache: {}",
                report.elapsed, report.completed_depth, report.cache_size
            );
        }

        if board.make_move(best_move.0, best_move.1) {
            stats.record_search(player, &report);
        } else {
            stats.outcome = board.outcome();
            (stats.x_boards, stats.o_boards) = board.local_board_counts();
            if verbose {
                println!("Erreur: l'IA a produit un coup illegal.");
                print_game_result(&board);
            }
            return stats;
        }
    }
}

fn read_game_mode() -> io::Result<GameMode> {
    loop {
        print!(
            "Mode de jeu ? (1: joueur contre IA, 2: joueur contre joueur, 3: IA contre IA, 4: benchmark, 5: tournoi): "
        );
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            return Ok(GameMode::HumanVsAi);
        }

        match input.trim() {
            "1" => return Ok(GameMode::HumanVsAi),
            "2" => return Ok(GameMode::HumanVsHuman),
            "3" => return Ok(GameMode::AiVsAi),
            "4" => return Ok(GameMode::Benchmark),
            "5" => return Ok(GameMode::Tournament),
            _ => println!("Choix invalide. Tapez 1, 2, 3, 4 ou 5."),
        }
    }
}

fn read_ai_player() -> io::Result<Player> {
    loop {
        print!("Qui commence ? (1: IA, 2: joueur): ");
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            return Ok(Player::O);
        }

        match input.trim() {
            "1" => return Ok(Player::X),
            "2" => return Ok(Player::O),
            _ => println!("Choix invalide. Tapez 1 pour l'IA ou 2 pour le joueur."),
        }
    }
}

fn read_human_move(board: &mut Board) -> io::Result<bool> {
    loop {
        print!("Entrez colonne ligne (ex: 5 5): ");
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            return Ok(false);
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() != 2 {
            println!("Format invalide. Utilise: colonne ligne, avec deux valeurs entre 1 et 9.");
            continue;
        }

        let parsed_column = parts[0].parse::<usize>();
        let parsed_row = parts[1].parse::<usize>();

        if let (Ok(column), Ok(row)) = (parsed_column, parsed_row) {
            if let Some(candidate) = coordinates_to_move(column, row) {
                let player = board.current_player();
                if board.make_move(candidate.0, candidate.1) {
                    println!("Joueur {player} joue: colonne {column} ligne {row}");
                    return Ok(true);
                }

                println!("Coup invalide: case occupee ou grille imposee non respectee.");
                continue;
            }
        }

        println!("Coordonnees invalides. La colonne et la ligne doivent etre entre 1 et 9.");
    }
}

fn read_compact_human_move(board: &mut Board) -> io::Result<bool> {
    loop {
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            return Ok(false);
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() != 2 {
            eprint!("format colonne ligne> ");
            io::stderr().flush()?;
            continue;
        }

        let parsed_column = parts[0].parse::<usize>();
        let parsed_row = parts[1].parse::<usize>();

        if let (Ok(column), Ok(row)) = (parsed_column, parsed_row) {
            if let Some(candidate) = coordinates_to_move(column, row) {
                if board.make_move(candidate.0, candidate.1) {
                    return Ok(true);
                }
            }
        }

        eprint!("coup invalide> ");
        io::stderr().flush()?;
    }
}

fn read_usize_with_default(prompt: &str, default: usize) -> io::Result<usize> {
    loop {
        print!("{prompt} [{default}]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            return Ok(default);
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(default);
        }

        if let Ok(value) = trimmed.parse::<usize>() {
            if value > 0 {
                return Ok(value);
            }
        }

        println!("Valeur invalide. Entrez un nombre entier strictement positif.");
    }
}

fn read_u64_with_default(prompt: &str, default: u64) -> io::Result<u64> {
    loop {
        print!("{prompt} [{default}]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            return Ok(default);
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(default);
        }

        if let Ok(value) = trimmed.parse::<u64>() {
            if value > 0 {
                return Ok(value);
            }
        }

        println!("Valeur invalide. Entrez un nombre entier strictement positif.");
    }
}

fn outcome_winner(outcome: GameOutcome) -> Option<Player> {
    match outcome {
        GameOutcome::MacroWin(player) => Some(player),
        GameOutcome::TieBreakWin { winner, .. } => Some(winner),
        GameOutcome::Draw { .. } | GameOutcome::Ongoing => None,
    }
}

fn average_ms(total: Duration, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        total.as_secs_f64() * 1000.0 / count as f64
    }
}

fn average_depth(total: u64, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    }
}

fn print_ai_game_stats(stats: &GameStats) {
    println!("Statistiques IA contre IA:");
    println!(
        "Coups: {} | temps moyen X: {:.2} ms | temps moyen O: {:.2} ms",
        stats.moves,
        average_ms(stats.x_time, stats.x_moves),
        average_ms(stats.o_time, stats.o_moves)
    );
    println!(
        "Profondeur moyenne X: {:.2} | profondeur moyenne O: {:.2}",
        average_depth(stats.x_depth_total, stats.x_moves),
        average_depth(stats.o_depth_total, stats.o_moves)
    );
}

fn print_benchmark_game(index: usize, main_player: Player, stats: &GameStats) {
    let result = match outcome_winner(stats.outcome) {
        Some(winner) if winner == main_player => "victoire IA principale",
        Some(_) => "defaite IA principale",
        None => "nul",
    };

    println!(
        "Partie {index}: {result} | IA principale={main_player} | coups={} | grilles X/O={}/{}",
        stats.moves, stats.x_boards, stats.o_boards
    );
}

fn print_benchmark_summary(games: usize, stats: &BenchmarkStats) {
    println!("\nRESULTAT BENCHMARK");
    println!(
        "IA principale: {}V / {}D / {}N sur {games} parties",
        stats.main_wins, stats.opponent_wins, stats.draws
    );
    println!(
        "Victoires par symbole: X = {}, O = {}",
        stats.x_wins, stats.o_wins
    );
    println!(
        "IA principale avec X: {}V / {}D / {}N",
        stats.main_as_x_wins, stats.main_as_x_losses, stats.main_as_x_draws
    );
    println!(
        "IA principale avec O: {}V / {}D / {}N",
        stats.main_as_o_wins, stats.main_as_o_losses, stats.main_as_o_draws
    );
    println!(
        "Coups moyens par partie: {:.2}",
        stats.total_moves as f64 / games as f64
    );
    println!(
        "Temps moyen par coup: IA principale = {:.2} ms | adversaire = {:.2} ms",
        average_ms(stats.main_time, stats.main_moves),
        average_ms(stats.opponent_time, stats.opponent_moves)
    );
    println!(
        "Profondeur moyenne: IA principale = {:.2} | adversaire = {:.2}",
        average_depth(stats.main_depth_total, stats.main_moves),
        average_depth(stats.opponent_depth_total, stats.opponent_moves)
    );
}

fn print_compact_game_result(board: &Board) {
    match board.outcome() {
        GameOutcome::MacroWin(player) => println!("FIN {player}"),
        GameOutcome::TieBreakWin { winner, .. } => println!("FIN {winner}"),
        GameOutcome::Draw { .. } => println!("FIN NUL"),
        GameOutcome::Ongoing => println!("FIN INCONNUE"),
    }
}

fn print_game_result(board: &Board) {
    let (x_boards, o_boards) = board.local_board_counts();
    println!("FIN DE PARTIE");

    match board.outcome() {
        GameOutcome::MacroWin(player) => {
            println!("Victoire de {player} par alignement de macro-grilles.");
            println!("Petites grilles gagnees: X = {x_boards}, O = {o_boards}");
        }
        GameOutcome::TieBreakWin {
            winner,
            x_boards,
            o_boards,
        } => {
            println!("Plateau complet sans alignement macro.");
            println!("Victoire de {winner} au departage des petites grilles.");
            println!("Petites grilles gagnees: X = {x_boards}, O = {o_boards}");
        }
        GameOutcome::Draw { x_boards, o_boards } => {
            println!("Match nul complet.");
            println!("Petites grilles gagnees: X = {x_boards}, O = {o_boards}");
        }
        GameOutcome::Ongoing => {
            println!("La partie n'est pas terminee.");
        }
    }
}
