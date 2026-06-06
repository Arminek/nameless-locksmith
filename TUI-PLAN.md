# Project status & TUI plan

Handoff note for continuing development (e.g. on the MacBook). Delete once the TUI ships.

## Where things stand

- **Solver + CLI** (`src/main.rs`, binary `locks`) is done and working: std-only Rust,
  BFS over 7⁶ plate states, commands `list / show / find / template / solve / help`,
  `--save` and `--file` options.
- **History** (`history-of-locks.md`) holds 5 solved locks + 1 empty placeholder.
  All 5 re-verified solvable via the solver:
  | Lock | Clicks (optimal) |
  |------|------------------|
  | 1 Chest above Cavalorn's | 35 |
  | 2 Cave | 41 |
  | 3 Door to tower | 57 |
  | 4 First chest in tower | 32 |
  | 5 Second chest in tower | 52 |
- **Recent fixes:** legacy Polish-translation `p` direction in locks 2 & 3 corrected to `r`;
  their solutions replaced with BFS-optimal ones (Cave 43→41, Door 61→57).
- **Release CI** (`.github/workflows/release.yml`) builds Linux/macOS(x64+arm64)/Windows
  binaries on pushing a `v*` tag.
- **README.md** added (capture convention, input format, commands).

## Pending: push + release (do this first)

Remote is set to HTTPS. Git Credential Manager will prompt via browser on first push.

```sh
git push -u origin main
git tag -a v0.1.0 -m "v0.1.0"
git push origin v0.1.0        # triggers the release workflow
```

(SSH was abandoned on Windows: the `~/.ssh/id_ed25519` key is passphrase-encrypted and
the agent wasn't loaded. On macOS, SSH will likely be easier if you prefer to switch back.)

## TUI plan (ratatui) — the next feature

Goal: nicer terminal UX via https://ratatui.rs/. Adds `ratatui` + `crossterm` deps
(no longer std-only — needs crates.io access). Keep the existing CLI subcommands intact;
launch the TUI when `locks` is run with **no args**.

Three views to build:
1. **Browse history** — scrollable list of locks (arrow keys) with a live filter box
   (replaces `find`). Enter opens detail (rules + start + solution).
2. **Solve** — an in-place form: 6 rule rows + start positions, then run BFS and show result.
3. **Step-through** — walk a solution one move at a time, highlighting the current step and
   rendering the evolving plate positions [1..7] per tumbler (most useful while at the
   in-game lock).

Suggested structure: keep solver/parsing logic in `main.rs` (or split into a `lib.rs` /
modules) so both the CLI and TUI call the same `solve`, `parse_history`, `parse_input`.
Reuse `Lock`, `rule_to_delta`, `build_matrix`, `solve`.

Open scope question to confirm with the user: build all three views, or start with just
the history browser?
