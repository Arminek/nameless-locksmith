// locks.rs — CLI to maintain the Gothic Remake lock history and solve new locks.
//
// Build:  rustc -O locks.rs -o locks.exe
// Usage:
//   locks list                      list every lock in the history
//   locks show <index|substring>    print full details of matching lock(s)
//   locks find <query>              search lock names (case-insensitive)
//   locks solve <input|->  [opts]   solve a lock from a file (or stdin via "-")
//        --save "<name>"            append the solved lock to the history file
//   global:  --file <path>          history file (default: history-of-locks.md)
//
// Conventions (plate frame): track positions 1..7, goal = every plate at 4.
//   Pressing [D] on a tumbler moves its own plate -1, plus its rule effects.
//   In a rule, `r` = that plate -1, `l` = that plate +1. [A] is the inverse of [D].

use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs;
use std::io::Read;
use std::process::exit;

const GOAL: [i32; 6] = [4, 4, 4, 4, 4, 4];
const DEFAULT_FILE: &str = "history-of-locks.md";

#[derive(Clone)]
struct Lock {
    name: String,
    rules: [String; 6], // raw text per tumbler, e.g. "3r, 6l" or "-"
    start: Option<[i32; 6]>,
    solution: Vec<String>, // grouped step lines, or a single note line
}

// ---------- rules -> delta matrix ----------

// D-press delta for one tumbler from its rule text. `idx` is the tumbler (0..5).
fn rule_to_delta(idx: usize, text: &str) -> Result<[i32; 6], String> {
    let mut d = [0i32; 6];
    d[idx] = -1; // own plate always moves -1 on a [D] press
    let t = text.trim();
    if t == "-" || t.is_empty() {
        return Ok(d);
    }
    for tok in t.split(',') {
        let tok = tok.trim();
        if tok.is_empty() {
            continue;
        }
        let dir = tok.chars().last().unwrap();
        let num: usize = tok[..tok.len() - 1]
            .trim()
            .parse()
            .map_err(|_| format!("bad token '{}' in rule {}", tok, idx + 1))?;
        if !(1..=6).contains(&num) {
            return Err(format!("plate {} out of range in rule {}", num, idx + 1));
        }
        let delta = match dir {
            'r' => -1,
            'l' => 1,
            other => {
                return Err(format!(
                    "unknown direction '{}' in rule {} (only r/l supported)",
                    other,
                    idx + 1
                ))
            }
        };
        d[num - 1] += delta;
    }
    Ok(d)
}

fn build_matrix(rules: &[String; 6]) -> Result<[[i32; 6]; 6], String> {
    let mut m = [[0i32; 6]; 6];
    for i in 0..6 {
        m[i] = rule_to_delta(i, &rules[i])?;
    }
    Ok(m)
}

// ---------- solver ----------

type State = [i32; 6];

struct Solution {
    total: usize,
    steps: Vec<(usize, char, usize)>, // (tumbler 1..6, key, count)
}

fn solve(start: State, mat: &[[i32; 6]; 6]) -> Option<Solution> {
    let mut prev: HashMap<State, (State, usize, char)> = HashMap::new();
    let mut queue: VecDeque<State> = VecDeque::new();
    queue.push_back(start);
    prev.insert(start, (start, usize::MAX, ' '));

    let mut reached = start == GOAL;
    while let Some(cur) = queue.pop_front() {
        if cur == GOAL {
            reached = true;
            break;
        }
        for t in 0..6 {
            for (key, sgn) in [('D', 1i32), ('A', -1i32)] {
                let mut next = cur;
                let mut ok = true;
                for i in 0..6 {
                    next[i] += sgn * mat[t][i];
                    if next[i] < 1 || next[i] > 7 {
                        ok = false;
                        break;
                    }
                }
                if !ok || prev.contains_key(&next) {
                    continue;
                }
                prev.insert(next, (cur, t, key));
                queue.push_back(next);
            }
        }
    }
    if !reached {
        return None;
    }

    let mut path: Vec<(usize, char)> = Vec::new();
    let mut cur = GOAL;
    loop {
        let (p, t, key) = prev[&cur];
        if t == usize::MAX {
            break;
        }
        path.push((t + 1, key));
        cur = p;
    }
    path.reverse();

    let total = path.len();
    let mut steps: Vec<(usize, char, usize)> = Vec::new();
    for &(t, key) in &path {
        if let Some(last) = steps.last_mut() {
            if last.0 == t && last.1 == key {
                last.2 += 1;
                continue;
            }
        }
        steps.push((t, key, 1));
    }
    Some(Solution { total, steps })
}

// ---------- history parsing ----------

fn parse_int_array(s: &str) -> Option<[i32; 6]> {
    let l = s.find('[')?;
    let r = s.find(']')?;
    let inner = &s[l + 1..r];
    let parts: Vec<i32> = inner
        .split(',')
        .filter_map(|x| x.trim().parse().ok())
        .collect();
    if parts.len() == 6 {
        let mut a = [0i32; 6];
        a.copy_from_slice(&parts);
        Some(a)
    } else {
        None
    }
}

// Collect the lines inside the first ``` fence at/after `from`.
fn fence_after(body: &[&str], from: usize) -> Option<Vec<String>> {
    let mut j = from;
    while j < body.len() && body[j].trim() != "```" {
        j += 1;
    }
    if j >= body.len() {
        return None;
    }
    j += 1;
    let mut content = Vec::new();
    while j < body.len() && body[j].trim() != "```" {
        content.push(body[j].to_string());
        j += 1;
    }
    Some(content)
}

fn parse_section(name: String, body: &[&str]) -> Lock {
    let mut rules: [String; 6] = Default::default();
    let mut start = None;
    let mut solution = Vec::new();

    let find = |needle: &str| body.iter().position(|l| l.contains(needle));

    if let Some(ri) = find("**Rules") {
        if let Some(content) = fence_after(body, ri) {
            for line in content {
                let line = line.trim();
                if let Some(colon) = line.find(':') {
                    if let Ok(n) = line[..colon].trim().parse::<usize>() {
                        if (1..=6).contains(&n) {
                            rules[n - 1] = line[colon + 1..].trim().to_string();
                        }
                    }
                }
            }
        }
    }
    if let Some(si) = find("**Start") {
        start = parse_int_array(body[si]);
    }
    if let Some(soli) = find("**Solution") {
        if let Some(content) = fence_after(body, soli) {
            solution = content.into_iter().filter(|l| !l.trim().is_empty()).collect();
        } else {
            // inline note after the marker
            if let Some(colon) = body[soli].find(':') {
                let note = body[soli][colon + 1..].trim().trim_start_matches("**").trim();
                if !note.is_empty() {
                    solution.push(note.to_string());
                }
            }
        }
    }

    Lock {
        name,
        rules,
        start,
        solution,
    }
}

fn parse_history(text: &str) -> Vec<Lock> {
    let lines: Vec<&str> = text.lines().collect();
    let sec_starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.starts_with("## "))
        .map(|(i, _)| i)
        .collect();
    let mut locks = Vec::new();
    for (k, &s) in sec_starts.iter().enumerate() {
        let end = if k + 1 < sec_starts.len() {
            sec_starts[k + 1]
        } else {
            lines.len()
        };
        let name = lines[s][3..].trim().to_string();
        locks.push(parse_section(name, &lines[s + 1..end]));
    }
    locks
}

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

// ---------- solve input parsing ----------

fn parse_input(text: &str) -> Result<(String, [String; 6], [i32; 6]), String> {
    let mut name = String::from("unnamed");
    let mut rules: [String; 6] = Default::default();
    let mut start: Option<[i32; 6]> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let lower = line.to_lowercase();
        if let Some(rest) = lower.strip_prefix("name:") {
            let _ = rest;
            name = line[line.find(':').unwrap() + 1..].trim().to_string();
            continue;
        }
        if lower.starts_with("rules") || lower.starts_with("start") {
            continue;
        }
        if line.contains('[') {
            if let Some(s) = parse_int_array(line) {
                start = Some(s);
            }
            continue;
        }
        if let Some(colon) = line.find(':') {
            if let Ok(n) = line[..colon].trim().parse::<usize>() {
                if (1..=6).contains(&n) {
                    rules[n - 1] = line[colon + 1..].trim().to_string();
                }
            }
        }
    }
    let start = start.ok_or("input is missing a Start line like [5,3,6,7,2,7]")?;
    Ok((name, rules, start))
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

fn solution_lines(sol: &Solution) -> Vec<String> {
    sol.steps
        .iter()
        .map(|(t, k, n)| format!("{}: {}x {}", t, n, k))
        .collect()
}

fn append_lock(file: &str, name: &str, rules: &[String; 6], start: &[i32; 6], sol: &Solution) {
    let mut out = String::new();
    out.push_str(&format!("\n## {}\n\n", name));
    out.push_str("**Rules:**\n```\n");
    for i in 0..6 {
        let r = if rules[i].is_empty() { "-" } else { &rules[i] };
        out.push_str(&format!("{}: {}\n", i + 1, r));
    }
    out.push_str("```\n\n");
    out.push_str(&format!(
        "**Start:** `[{}]`\n\n",
        start.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
    ));
    out.push_str(&format!(
        "**Solution:** ({} clicks — solved by locks.rs)\n```\n",
        sol.total
    ));
    for line in solution_lines(sol) {
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("```\n");

    let mut existing = fs::read_to_string(file).unwrap_or_default();
    existing.push_str(&out);
    fs::write(file, existing).unwrap_or_else(|e| {
        eprintln!("cannot write {}: {}", file, e);
        exit(1);
    });
    println!("Saved \"{}\" to {}", name, file);
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
  locks list                      List every lock in the history ([✓] = solved).\n\
  locks show <index|substring>    Print full details (rules, start, solution).\n\
                                  e.g.  locks show 5      locks show \"tower\"\n\
  locks find <query>              Search lock names (case-insensitive).\n\
  locks template                  Print a ready-to-fill input file for `solve`.\n\
  locks solve <input|->  [opts]   Solve a lock from a file (or stdin via \"-\").\n\
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

    if args.is_empty() {
        usage();
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
                        append_lock(&file, &nm, &rules, &start, &sol);
                    } else {
                        println!("(tip: add  --save \"<name>\"  to record this lock in {})", file);
                    }
                }
            }
        }
        _ => usage(),
    }
}
