// main.rs — `locks` CLI to maintain the Gothic Remake lock history and solve locks.
//
// Build:  cargo build --release   (binary at target/release/locks)
// Usage:
//   locks                            launch the interactive TUI (no args)
//   locks list                       list every lock in the history
//   locks show <index|substring>     print full details of matching lock(s)
//   locks find <query>               search lock names (case-insensitive)
//   locks template                   print a ready-to-fill input for `solve`
//   locks solve <input|->  [opts]    solve a lock from a file (or stdin via "-")
//        --save "<name>"             append the solved lock to the history file
//   global:  --file <path>           history file (default: history-of-locks.md)
//
// Core solver/parsing lives in the library crate (src/lib.rs); the TUI in src/tui.rs.

use std::env;
use std::fs;
use std::io::Read;
use std::process::exit;

use nameless_locksmith::{
    append_lock, build_matrix, parse_history, parse_input, remove_lock_from_file, solution_lines,
    solve, Lock, DEFAULT_FILE,
};

mod tui;

// ---------- formatting ----------

fn print_lock(l: &Lock) {
    println!("## {}", l.name);
    println!("Rules:");
    for i in 0..6 {
        let r = if l.rules[i].is_empty() { "-" } else { &l.rules[i] };
        println!("  {}: {}", i + 1, r);
    }
    match l.start {
        Some(s) => println!(
            "Start: [{}]",
            s.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
        ),
        None => println!("Start: (none)"),
    }
    if l.solution.is_empty() {
        println!("Solution: (none)");
    } else {
        println!("Solution:");
        for step in &l.solution {
            println!("  {}", step);
        }
    }
}

fn read_source(path: &str) -> String {
    if path == "-" {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s).expect("read stdin");
        s
    } else {
        fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("cannot read {}: {}", path, e);
            exit(1);
        })
    }
}

// ---------- CLI ----------

const TEMPLATE: &str = "\
# ---------------------------------------------------------------------------
# Lock input for `locks solve`.  Fill this in by reading the lock IN-GAME.
#
# HOW TO CAPTURE (plate frame — watch the PLATES, they are easy to see):
#   * Press [D] on a tumbler  -> its plate slides RIGHT.
#   * For every OTHER plate that moves, note the direction:
#         r = that plate slides RIGHT      l = that plate slides LEFT
#         -  (a dash) = nothing else moves
#   * IMPORTANT: if a plate is jammed at a wall, a [D] press can be BLOCKED,
#     which HIDES dependencies. Nudge plates toward the middle first so every
#     dependency has room to move and be seen.
#
# Lines starting with '#' are ignored. Spaces are optional.
# ---------------------------------------------------------------------------

Name: my new lock

Rules:
1: 3r, 6l
2: -
3: 1r, 4l, 6r
4: 2r, 5r, 6l
5: -
6: 3l

# Start = the current position (1-7) of each plate, tumblers 1..6, left to right.
# Goal is always to center every plate at 4.
Start:
[5, 3, 6, 7, 2, 7]
";

fn usage() -> ! {
    eprintln!("{}", help_text());
    exit(2);
}

fn help_text() -> String {
    format!(
        "locks — helper for picking complex locks in Gothic Remake\n\
\n\
WHAT IT DOES\n\
  Browse your solved-lock history and compute the shortest wall-safe key\n\
  sequence for a new lock (breadth-first search over all 7^6 plate states).\n\
\n\
COMMANDS\n\
  locks                           Launch the interactive TUI (browse / solve / step).\n\
  locks list                      List every lock in the history ([✓] = solved).\n\
  locks show <index|substring>    Print full details (rules, start, solution).\n\
                                  e.g.  locks show 5      locks show \"tower\"\n\
  locks find <query>              Search lock names (case-insensitive).\n\
  locks template                  Print a ready-to-fill input file for `solve`.\n\
  locks solve <input|->  [opts]   Solve a lock from a file (or stdin via \"-\").\n\
  locks remove <index|substring>  Delete a lock from the history (alias: rm).\n\
  locks help                      Show this help.\n\
\n\
SOLVE OPTIONS\n\
  --save \"<name>\"                 Append the solved lock to the history file.\n\
\n\
GLOBAL OPTIONS\n\
  --file <path>                   History file to use (default: {def}).\n\
\n\
INPUT FORMAT (for `solve`)\n\
  A small text block.  Get a starter with:  locks template > lock.txt\n\
\n\
      Name: <optional label>\n\
      Rules:\n\
      1: 3r, 6l        <- pressing [D] on tumbler 1 moves plate 3 right, plate 6 left\n\
      2: -             <- '-' means no other plate moves\n\
      3: 1r, 4l, 6r\n\
      4: 2r, 5r, 6l\n\
      5: -\n\
      6: 3l\n\
      Start:\n\
      [5, 3, 6, 7, 2, 7]   <- current position 1..7 of plates 1..6\n\
\n\
  Direction letters:  r = plate slides RIGHT,  l = plate slides LEFT.\n\
  Goal is always to center every plate at position 4.\n\
\n\
OUTPUT\n\
  A grouped key sequence to type in-game, e.g.  `4: 2x D` = press [D] twice on tumbler 4.\n\
  [D] slides a plate right, [A] slides it left; [W]/[S] switch between tumblers.\n\
\n\
EXAMPLES\n\
  locks                       # interactive TUI\n\
  locks list\n\
  locks show \"Second chest\"\n\
  locks template > lock.txt   # then edit lock.txt with your readings\n\
  locks solve lock.txt\n\
  locks solve lock.txt --save \"Vault behind the inn\"\n\
  type lock.txt | locks solve -      (PowerShell:  Get-Content lock.txt | locks solve -)\n",
        def = DEFAULT_FILE
    )
}

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();

    // pull global --file
    let mut file = DEFAULT_FILE.to_string();
    if let Some(p) = args.iter().position(|a| a == "--file") {
        if p + 1 >= args.len() {
            usage();
        }
        file = args[p + 1].clone();
        args.drain(p..=p + 1);
    }

    // No subcommand -> launch the interactive TUI.
    if args.is_empty() {
        if let Err(e) = tui::run(&file) {
            eprintln!("tui error: {}", e);
            exit(1);
        }
        return;
    }
    let cmd = args.remove(0);

    match cmd.as_str() {
        "help" | "--help" | "-h" => {
            println!("{}", help_text());
        }
        "template" => {
            print!("{}", TEMPLATE);
        }
        "list" => {
            let locks = parse_history(&read_source(&file));
            for (i, l) in locks.iter().enumerate() {
                let solved = if l.solution.is_empty() { " " } else { "✓" };
                println!("{:>2}. [{}] {}", i + 1, solved, l.name);
            }
        }
        "find" => {
            if args.is_empty() {
                usage();
            }
            let q = args[0].to_lowercase();
            let locks = parse_history(&read_source(&file));
            let mut any = false;
            for (i, l) in locks.iter().enumerate() {
                if l.name.to_lowercase().contains(&q) {
                    println!("{:>2}. {}", i + 1, l.name);
                    any = true;
                }
            }
            if !any {
                println!("no lock matches \"{}\"", args[0]);
            }
        }
        "show" => {
            if args.is_empty() {
                usage();
            }
            let locks = parse_history(&read_source(&file));
            let q = &args[0];
            let mut shown = false;
            if let Ok(idx) = q.parse::<usize>() {
                if idx >= 1 && idx <= locks.len() {
                    print_lock(&locks[idx - 1]);
                    shown = true;
                }
            }
            if !shown {
                let ql = q.to_lowercase();
                for l in locks.iter().filter(|l| l.name.to_lowercase().contains(&ql)) {
                    print_lock(l);
                    println!();
                    shown = true;
                }
            }
            if !shown {
                println!("no lock matches \"{}\"", q);
            }
        }
        "remove" | "rm" | "delete" => {
            if args.is_empty() {
                usage();
            }
            let q = &args[0];
            let locks = parse_history(&read_source(&file));
            // Resolve the argument to a single lock index (0-based).
            let idx = if let Ok(n) = q.parse::<usize>() {
                if n >= 1 && n <= locks.len() {
                    Some(n - 1)
                } else {
                    eprintln!("no lock at position {} (history has {})", n, locks.len());
                    exit(1);
                }
            } else {
                let ql = q.to_lowercase();
                let matches: Vec<usize> = locks
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| l.name.to_lowercase().contains(&ql))
                    .map(|(i, _)| i)
                    .collect();
                match matches.as_slice() {
                    [] => {
                        println!("no lock matches \"{}\"", q);
                        exit(1);
                    }
                    [one] => Some(*one),
                    many => {
                        eprintln!("\"{}\" matches {} locks — be more specific:", q, many.len());
                        for &i in many {
                            eprintln!("  {}. {}", i + 1, locks[i].name);
                        }
                        exit(1);
                    }
                }
            };
            match remove_lock_from_file(&file, idx.unwrap()) {
                Ok(name) => println!("Removed \"{}\" from {}", name, file),
                Err(e) => {
                    eprintln!("{}", e);
                    exit(1);
                }
            }
        }
        "solve" => {
            if args.is_empty() {
                eprintln!(
                    "solve needs an input file (or \"-\" for stdin).\n\
                     Get a starter template with:\n\
                     \x20   locks template > lock.txt\n\
                     then edit lock.txt and run:\n\
                     \x20   locks solve lock.txt\n"
                );
                exit(2);
            }
            let input_path = args.remove(0);
            let mut save_name: Option<String> = None;
            if let Some(p) = args.iter().position(|a| a == "--save") {
                if p + 1 >= args.len() {
                    usage();
                }
                save_name = Some(args[p + 1].clone());
            }

            let text = read_source(&input_path);
            let (name, rules, start) = parse_input(&text).unwrap_or_else(|e| {
                eprintln!("input error: {}", e);
                exit(1);
            });
            let mat = build_matrix(&rules).unwrap_or_else(|e| {
                eprintln!("rule error: {}", e);
                exit(1);
            });
            // Echo back what was understood, so the user can confirm the capture.
            println!("Lock: {}", name);
            println!("Rules (interpreted):");
            for i in 0..6 {
                let r = if rules[i].is_empty() { "-" } else { &rules[i] };
                println!("  {}: {}", i + 1, r);
            }
            println!(
                "Start:  [{}]",
                start.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
            );
            println!("Goal:   [4, 4, 4, 4, 4, 4]   (center every plate)");
            println!();

            match solve(start, &mat) {
                None => {
                    eprintln!(
                        "NO SOLUTION — the goal [4,4,4,4,4,4] cannot be reached from this start\n\
                         without a plate hitting a wall (position <1 or >7).\n\
                         This usually means a rule was mis-read while a plate sat at a wall\n\
                         (a blocked [D] press hides dependencies). Re-capture with the plates\n\
                         nudged toward the middle, then try again."
                    );
                    exit(1);
                }
                Some(sol) => {
                    println!(
                        "SOLUTION — {} clicks total, {} grouped steps:",
                        sol.total,
                        sol.steps.len()
                    );
                    println!("(read top to bottom; [D]=plate right, [A]=plate left, [W]/[S]=switch tumbler)");
                    println!("------------------------------------------------------------");
                    for line in solution_lines(&sol) {
                        println!("  {}", line);
                    }
                    println!("------------------------------------------------------------");
                    if let Some(n) = save_name {
                        let nm = if n.is_empty() { name } else { n };
                        match append_lock(&file, &nm, &rules, &start, &sol) {
                            Ok(()) => println!("Saved \"{}\" to {}", nm, file),
                            Err(e) => {
                                eprintln!("{}", e);
                                exit(1);
                            }
                        }
                    } else {
                        println!("(tip: add  --save \"<name>\"  to record this lock in {})", file);
                    }
                }
            }
        }
        _ => usage(),
    }
}
