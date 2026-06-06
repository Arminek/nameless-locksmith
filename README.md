# nameless-locksmith

A command-line **and terminal-UI** solver and history manager for the **lock-picking minigame
in Gothic Remake**.

Each lock has 6 interconnected tumblers (plates) that slide along a 1–7 track. Moving one
tumbler forces others to move, and no plate may ever fall below 1 or past 7. The goal is to
center every plate at position **4**. `locks` finds the **shortest wall-safe key sequence**
via breadth-first search over all 7⁶ plate states, and keeps a log of every lock you've solved.

## Interactive TUI

Run `locks` with no arguments to open the terminal UI (built with
[ratatui](https://ratatui.rs/)). On startup it asks for a language — **English, Polski, Deutsch,
Русский, Українська, Español, Português, Français** — then gives you three views:

- **Browse** — a filterable list of your solved locks with a detail pane. `d` deletes the
  selected lock (with a y/n confirm).
- **Solve** — an in-place form (6 rules + start) that runs the solver and shows the result.
- **Step** — walk a solution one click at a time. The six plates are stacked and **aligned**, so
  any tumblers at the same position line up; the plate slides while the pin stays put (as in the
  game), each pin turning green as it seats on hole 4. The lock is open when all six form one
  vertical column at the centre. A big panel shows the current move and the steps scroll alongside:

```
            ▼  4                       ┌ Current move (9/31) ──────┐
 align every pin on hole 4             │        █▀█ ▄█  █▀▖        │
  6         ▕ ○ ◉ ○ ◌ ○ ○ ○ ▏ 2        │         ▀█  █  █ █        │
  5           ▕ ◉ ○ ○ ◌ ○ ○ ○ 1        │        █▄█ ▄█▄ █▄▘        │
  4         ▕ ○ ◉ ○ ◌ ○ ○ ○ ▏ 2        │           D  →           │
▶ 3       ▕ ○ ○ ◉ ◌ ○ ○ ○ ▏   3        └───────────────────────────┘
  2     ▕ ○ ○ ○ ◉ ○ ○ ○ ▏     ✓        ┌ Steps (9/31) ─────────────┐
  1           ▕ ◉ ○ ○ ◌ ○ ○ ○ 1        │ ✓ 1   1× A                │
                                       │ ▶ 3   1× D                │
 click 8 / 35   · 1/6 pins on 4        │   5   1× A   …            │
```

All the `locks <subcommand>` commands below still work unchanged.

Adding a UI language is just dropping a `key = value` file in [`src/i18n/`](src/i18n/) and
registering one row in `LANGUAGES` (see [`src/i18n/en.txt`](src/i18n/en.txt)); missing keys fall
back to English, and a test enforces that every catalog has the full key set.

## Install

Grab a prebuilt binary for your OS from the [Releases](https://github.com/Arminek/nameless-locksmith/releases)
page (Linux, macOS Intel/Apple Silicon, Windows), or build from source:

```sh
cargo build --release
# binary at target/release/locks (locks.exe on Windows)
```

The core solver is std-only Rust; the TUI adds `ratatui` + `crossterm`. The step view uses
24-bit color, so a truecolor terminal renders the plate shading best.

## Quick start

```sh
locks                         # launch the interactive TUI
locks list                    # list every lock in the history
locks show "Second chest"     # show one lock's rules, start, and solution
locks find tower              # search lock names
locks template > lock.txt     # get a starter input file to fill in
locks solve lock.txt          # compute the optimal sequence for a new lock
```

## Capturing a lock in-game

You read the lock by **watching the plates** (easier to see than the small red pins):

1. Press **[D]** on a tumbler — its plate slides **right**.
2. For every *other* plate that moves, note the direction: `r` = right, `l` = left.
   Use `-` if nothing else moves.

> **Tip:** if a plate is jammed against a wall (position 1 or 7), a [D] press can be *blocked*,
> which **hides dependencies** — you'll only see the one plate that hit the wall. Nudge plates
> toward the middle first so every dependency has room to move and be observed.

Then record the current position (1–7) of each plate as the `Start` row.

### Input format

```
Name: my new lock
Rules:
1: 3r, 6l
2: -
3: 1r, 4l, 6r
4: 2r, 5r, 6l
5: -
6: 3l
Start:
[5, 3, 6, 7, 2, 7]
```

Run it:

```sh
locks solve lock.txt
locks solve lock.txt --save "Second chest in the tower"   # also append to history
cat lock.txt | locks solve -                              # read from stdin
```

## Reading the output

The solution is a grouped key sequence to type in-game:

```
4: 2x D    # press [D] twice on tumbler 4 (plate slides right)
2: 3x A    # press [A] three times on tumbler 2 (plate slides left)
```

- **[D]** slides a plate **right**, **[A]** slides it **left**.
- **[W]/[S]** switch between tumblers.

## Commands

| Command | Description |
|---|---|
| `locks list` | List every lock in the history (`[✓]` = solved). |
| `locks show <index\|substring>` | Print full details for a lock. |
| `locks find <query>` | Search lock names (case-insensitive). |
| `locks template` | Print a ready-to-fill input file for `solve`. |
| `locks solve <input\|->` | Solve a lock from a file (or stdin via `-`). |
| `locks remove <index\|substring>` | Delete a lock from the history (alias: `rm`). |
| `locks help` | Show full help. |

**Options:** `--save "<name>"` appends a solved lock to the history;
`--file <path>` selects a different history file (default `history-of-locks.md`).

## History

Solved locks live in [history-of-locks.md](history-of-locks.md) — rules, start positions, and
the step sequences for every lock encountered so far. It doubles as worked examples.

## License

MIT
