# Roadmap — lootscribe-slicer

## Execution & Git Protocol (MANDATORY)
- Execute **strictly ONE task per turn**. Never modify multiple unrelated modules in a single step.
- Keep every file edit atomic and concise.
- Before committing any step:
  1. Run `cargo check` and `cargo test`.
  2. Fix any warnings or compiler errors in the touched file.
- After passing checks:
  1. Run `git add <modified_files>`
  2. Run `git commit -m "<conventional_commit_message>"`
  3. Run `git push`
- Stop and report completion briefly after pushing each task.

---

## Phase 1: Fix Core Slicing & Pricing Bugs

- [ ] **Task 1.1 — Fix `pricing.rs` math & add unit tests**
  - Apply `fail_rate` to `custo` (production cost) before applying `profit` multiplier, rather than multiplying after Shopee fees.
  - Fix `margem` calculation to represent actual net profit margin (`(lucro_val / valor_venda) * 100.0`).
  - Add `#[cfg(test)]` unit tests covering the three Shopee fee tiers (`< 79.99`, `79.99..99.99`, `>= 99.99`).
  - Commit: `fix(pricing): apply fail_rate to base cost and fix net margin calculation`

- [ ] **Task 1.2 — Fix OrcaSlicer CLI invocation in `slicer.rs`**
  - Update `slice_with_orca` to resolve the `.json` profile files inside `config_dir` (machine, process, filament) and pass `--slice 0`, `--load-settings`, `--load-filaments`, `--outputdir`, and the positional input file path.
  - Handle missing `orca-slicer` binary gracefully with an actionable error message.
  - Commit: `fix(slicer): use valid OrcaSlicer CLI flags and profile loading`

- [ ] **Task 1.3 — Parse metrics directly from generated `.gcode` in `slicer.rs`**
  - Replace fragile stdout scraping with a parser that reads the generated `.gcode` file headers/footers (e.g., `; total filament weight [g] =` and `; estimated printing time (normal mode) =`).
  - Add unit tests in `slicer.rs` testing G-code comment parsing (`1h 23m 45s`, `45m 10s`, weights in grams).
  - Commit: `feat(slicer): parse print time and filament weight from generated gcode`

---

## Phase 2: Robustness & Workflow Features

- [ ] **Task 2.1 — Eliminate `unwrap()` panics and add `--json` output flag**
  - Replace `.unwrap()` calls in `main.rs` (`config.k1c_profile_dir().to_str()`) and `slicer.rs` (`file_stem()`) with proper `anyhow` error context.
  - Add `--json` flag to `cli.rs` (add `serde` + `serde_json` to `Cargo.toml`) so `lootscribe-slicer` can pipe structured results directly into `lootscribe_list` or `lootscribe-dashboard`.
  - Commit: `feat(cli): add --json output mode and remove unwrap panics`

- [ ] **Task 2.2 — Add batch slicing support (`--batch <DIR>`)**
  - Allow passing a directory of `.stl`/`.3mf` files in `cli.rs` and `main.rs` to slice and price multiple parts in sequence, printing a summary table at the end.
  - Commit: `feat(cli): add batch directory slicing and summary output`