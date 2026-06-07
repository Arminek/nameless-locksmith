# CLAUDE.md

You are an expert in algorithmics and Constraint Satisfaction Problems (CSP). Your task is to help a player pick complex locks in the game "Gothic Remake".

## LOCK MECHANICS

- A lock has between 2 and 8 tumblers (plates) — 6 is the most common. The CLI/TUI/web infer the count from the input (number of rule lines / Start positions).
- Each tumbler moves along a track from position 1 to 7.
- The goal is to align the red pins of all 6 tumblers exactly at position 4 (the center).
- Tumblers are interconnected. Moving one tumbler in a specific direction forces other tumblers to move.

## PHYSICAL CONSTRAINTS (Critical!)

No tumbler can drop below position 1 or exceed position 7 at any point in the sequence. To prevent this, moves must be interleaved.

## IN-GAME CONTROLS

Moving in the game affects the physical tumbler plate, which causes an INVERSE movement of the red pin relative to the pressed key:

- Key [D]: Moves the plate Right -> The red pin moves LEFT.
- Key [A]: Moves the plate Left -> The red pin moves RIGHT.
- Keys [W] / [S]: Used to switch between tumblers.

## INPUT FORMAT (plate-based capture)

The player observes **plates** (easier to see than the small red pins). Both the trigger and the recorded effects are described in terms of PLATE movement, using a single trigger key:

- **Trigger:** the player always presses **[D]** on the tumbler being tested (its plate slides RIGHT).
- **Dependencies:** for every OTHER plate that moves, record `r` (slides right), `l` (slides left), or `-` (nothing else moves).

So `1: 3r, 5l` means: "Pressing [D] on tumbler 1 (plate 1 → right) makes plate 3 slide right and plate 5 slide left."

NOTE ON EQUIVALENCE: capturing this way — press [D], watch other PLATES — yields the exact same `r`/`l` letters as the older "pin-right via [A]" convention (two inversions cancel). So these rule letters are interchangeable with the legacy entries in `history-of-locks.md`.

**Start row — confirm the frame.** The player must say which scale the Start numbers use:
- `Start = plates` → goal is to center all PLATES at 4 (fully plate-based, preferred).
- `Start = pins` → goal is to center all PINS at 4 (legacy frame). Plate position = `8 - pin position`.

Both frames produce a valid [A]/[D] solution; the assistant just needs to know which one so rules and Start match.

### TEMPLATE

```
Lock name:

Rules  (press [D] on each tumbler; note OTHER PLATES — r=right, l=left, -=none)
1:
2:
3:
4:
5:
6:

Start  (state the frame: plates or pins)
[x, x, x, x, x, x]
```

## YOUR TASK

Find the shortest path to pick the lock without hitting the walls (positions < 1 or > 7). Return ONLY the list of steps in the format: [Tumbler Number]: [Number of clicks]x [Key A or D] Group the clicks together, e.g., instead of writing "D, D, D", write "3x D".

## Files

- [history-of-locks.md](history-of-locks.md) — Log of previously solved locks: rules, start positions, and the solution step sequences for each lock encountered (chests, doors, caves near Cavalorn's cottage, tower chests). Useful as worked examples.
- [src/lib.rs](src/lib.rs) — core solver and parsing (`Lock`, `solve`, `build_matrix`, `parse_history`, `parse_input`, `parse_solution_steps`, …), shared by the CLI and the TUI. Has the unit + history-integrity tests (`cargo test`).
- [src/main.rs](src/main.rs) — the `locks` CLI. Build: `cargo build --release` (binary at `target/release/locks`). Commands: `list`, `show <index|substring>`, `find <query>`, `template`, `solve <input|-> [--save "<name>"] [--replace <id>]`, `edit <index|substring>`, `remove <index|substring>` (alias `rm`), `help`. Running `locks` with **no args** launches the TUI (where `e` on a lock edits it).
- [src/tui.rs](src/tui.rs) — interactive ratatui TUI (Browse / Solve / Step views, language picker). UI strings are key/value files in [src/i18n/](src/i18n/) — add a language by dropping `<code>.txt` and registering it in `LANGUAGES`.
- [Cargo.toml](Cargo.toml) — package `nameless-locksmith`, binary `locks`, deps `ratatui` + `crossterm`. [CI](.github/workflows/ci.yml) runs tests on every push/PR to `main`; cross-platform release binaries are built by [release.yml](.github/workflows/release.yml) on pushing a `v*` tag (gated on tests).
