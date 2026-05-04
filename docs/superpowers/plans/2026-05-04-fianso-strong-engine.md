# Fianso Strong Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create and push a GitHub-visible `fianso` branch with the strongest available Ultimate Tic Tac Toe player.

**Architecture:** Keep the existing CLI and board UI, then add a separate NNUE-style engine module imported from the reference repository's public `previous` branch. The CLI will maintain the current `Board` for validation/display and a synchronized bitboard `TicTacToe` state for engine search.

**Tech Stack:** Rust 2024, `rayon` for root parallel search, `bytemuck`/`once_cell` for quantized weight loading, public `databin/gen160_weights.bin`.

---

### Task 1: Engine Adapter Contract

**Files:**
- Modify: `src/main.rs`
- Create: `src/strong.rs`
- Create: `databin/.gitignore`
- Create: `databin/gen160_weights.bin`

- [x] Write failing tests for move conversion, legal move synchronization, and trained-weight move selection.
- [x] Import the reference bitboard core, move generator, NNUE network, and search.
- [x] Add a `StrongEngine` adapter that converts `(macro_idx, micro_idx)` moves to reference 0-80 squares.
- [x] Wire CLI AI move selection through `StrongEngine`, falling back to the existing heuristic only if the weight file is unavailable.
- [x] Run `cargo fmt`, `cargo test`, and `cargo build --release`.
- [ ] Commit the branch and push `fianso` to `origin`.
