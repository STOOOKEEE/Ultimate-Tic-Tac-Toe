# Ultimate Tic Tac Toe

AI agent for Ultimate Tic Tac Toe, built in Rust for the **Fondement de l'IA** course with Christophe Rodrigues.

Built by [Sofiane Ben Taleb](https://github.com/gamween), [Ramzy Chibani](https://github.com/DZ-Ramzy) and [Armand Séchon](https://github.com/STOOOKEEE).

---

## The Problem

Ultimate Tic Tac Toe is much harder to solve than classic Tic Tac Toe. The board contains nine local 3x3 boards arranged into one global 3x3 board. Each move not only places a symbol, but also constrains where the opponent must play next.

The assignment requires an AI that can play complete matches without bugs, respect the official rules, and make decisions quickly enough during timed battles. Brute force is not realistic: the game tree is too large, and the quality of the AI depends on the balance between search depth, evaluation quality and execution speed.

The subject also imposes an important constraint: the AI must not use a move dictionary. Every decision must be computed from the current board state.

## The Solution

This project implements a single strong AI method designed around time-limited adversarial search.

The AI uses:

- **Negamax / Minimax search** with **Alpha-Beta pruning**
- **Iterative deepening** to use the available time per move
- **Transposition table** to reuse already evaluated positions
- **Bitboard move generation** for fast legal move computation
- **NNUE-style evaluation weights** stored in `databin/gen160_weights.bin`

There is no fallback heuristic and no move dictionary. Legal moves are generated from the current board, then evaluated through search.

## Rules Implemented

The implementation follows the rules from `IA-5.pdf` and the standard Ultimate Tic Tac Toe rules described on [Wikipedia](https://en.wikipedia.org/wiki/Ultimate_tic-tac-toe):

- The game is played on a 9x9 grid, divided into nine 3x3 local boards.
- Players alternate turns. `X` starts.
- A move is entered as `column row`, both from 1 to 9.
- The previous move sends the opponent to the corresponding local board.
- If the forced local board is already won or full, the next player may play in any playable local board.
- A won or full local board cannot receive new moves.
- A player wins by winning three local boards aligned on the global board.
- If the board is full and no global alignment exists, the subject tie-break is applied: the player with the most won local boards wins. If both players won the same number of local boards, the game is a draw.

## How It Works

```text
User / Tournament input
   |
   v
CLI parser
   |
   v
Rule engine (src/game.rs)
   |-- validates moves
   |-- tracks forced local board
   |-- detects local wins, global wins and tie-breaks
   |
   v
AI wrapper (src/ai.rs)
   |
   v
Strong engine adapter (src/strong.rs)
   |
   v
Bitboard engine
   |-- move generation
   |-- Negamax / Alpha-Beta
   |-- iterative deepening
   |-- transposition table
   |-- NNUE evaluation
   |
   v
Best legal move: column row
```

## Why This Approach

The subject recommends Minimax with Alpha-Beta pruning, while warning that exploring the full tree is not practical. Wikipedia also highlights that Ultimate Tic Tac Toe is difficult for computers because it has a large branching factor and no simple universal evaluation function.

Our implementation addresses this with a time-limited search engine: iterative deepening searches progressively deeper and always keeps the best completed result. Alpha-Beta pruning removes branches that cannot improve the decision. The transposition table avoids recomputing equivalent states. The NNUE-style evaluator replaces a hand-written tactical score with a stronger position evaluation.

## Tech Stack

| Layer | Technology |
|------|------------|
| Language | Rust |
| Game interface | Terminal CLI |
| Search | Negamax / Minimax with Alpha-Beta pruning |
| Time management | Iterative deepening |
| Board representation | 9x9 rule board + bitboard engine adapter |
| Evaluation | NNUE-style weights |
| Parallelism | Rayon |
| Dependencies | `anyhow`, `bytemuck`, `colored`, `once_cell`, `rand`, `rayon` |

## Project Structure

```text
Ultimate-Tic-Tac-Toe/
├── src/
│   ├── main.rs        # Program entry point
│   ├── cli.rs         # Text interface and game modes
│   ├── game.rs        # Official game rules and board state
│   ├── coords.rs      # User coordinate conversion
│   ├── ai.rs          # Single public AI entry point
│   ├── strong.rs      # Adapter to the strong engine
│   ├── core.rs        # Bitboard game state
│   ├── movegen.rs     # Legal move generation
│   ├── search.rs      # Negamax, Alpha-Beta, iterative deepening
│   ├── network.rs     # NNUE-style evaluation network
│   └── constants.rs   # Bitboard masks and constants
├── databin/
│   └── gen160_weights.bin
├── IA-5.pdf
├── Cargo.toml
└── README.md
```

## Getting Started

### Prerequisites

Install Rust with [`rustup`](https://rustup.rs/).

### Run

```bash
cargo run --release
```

The `--release` flag is important. The AI is search-heavy, and debug builds are much slower.

### Menu

The program provides:

1. Player vs AI
2. Player vs player
3. AI vs AI
4. Benchmark
5. Tournament mode

Tournament mode prints only the AI move in the required `column row` format.

## Time Per Move

The default AI budget is 2 seconds per move in normal play and AI-vs-AI mode. Tournament mode asks for the move budget in milliseconds, with a default of 2000 ms.

A higher budget allows the iterative deepening search to complete deeper levels. For competition, 2000 ms is a strong default if the rules allow it. If the timing is strict, using a slightly lower value such as 1800 ms gives a safety margin.

## Tests

```bash
cargo test
```

The tests cover:

- coordinate conversion in `column row` order
- forced local board behavior
- free choice when the target board is won or full
- illegal moves in decided boards
- local board wins
- global board wins
- rejection of moves after a finished game
- tie-break on full board
- consistency between the rule board and the bitboard engine move generator

Additional checks used during cleanup:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
```

## Sources

- Course subject: `IA-5.pdf`
- Rules reference: https://en.wikipedia.org/wiki/Ultimate_tic-tac-toe

## Notes

The file `databin/gen160_weights.bin` is required. It contains evaluation weights for the strong engine. It is not a move dictionary: the AI still computes legal moves and searches from the current position at runtime.
