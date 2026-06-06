# nameless-locksmith

A tiny command-line solver and history manager for the **lock-picking minigame in Gothic Remake**.

Each lock has 6 interconnected tumblers (plates) that slide along a 1–7 track. Moving one
tumbler forces others to move, and no plate may ever fall below 1 or past 7. The goal is to
center every plate at position **4**. `locks` finds the **shortest wall-safe key sequence**
via breadth-first search over all 7⁶ plate states, and keeps a log of every lock you've solved.

## Install

Grab a prebuilt binary for your OS from the [Releases](https://github.com/Arminek/nameless-locksmith/releases)
page (Linux, macOS Intel/Apple Silicon, Windows), or build from source:

```sh
cargo build --release
# binary at target/release/locks (locks.exe on Windows)
```

No external dependencies — std-only Rust.

## Quick start

```sh
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
| `locks help` | Show full help. |

**Options:** `--save "<name>"` appends a solved lock to the history;
`--file <path>` selects a different history file (default `history-of-locks.md`).

## History

Solved locks live in [history-of-locks.md](history-of-locks.md) — rules, start positions, and
the step sequences for every lock encountered so far. It doubles as worked examples.

## License

MIT
