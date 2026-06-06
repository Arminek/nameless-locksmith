// lib.rs — core solver and history parsing for the Gothic Remake lock helper.
//
// Shared by the `locks` CLI (src/main.rs) and the TUI (src/tui.rs).
//
// Conventions (plate frame): track positions 1..7, goal = every plate at 4.
//   Pressing [D] on a tumbler moves its own plate -1, plus its rule effects.
//   In a rule, `r` = that plate -1, `l` = that plate +1. [A] is the inverse of [D].

use std::collections::{HashMap, VecDeque};
use std::fs;

pub const GOAL: [i32; 6] = [4, 4, 4, 4, 4, 4];
pub const DEFAULT_FILE: &str = "history-of-locks.md";

#[derive(Clone)]
pub struct Lock {
    pub name: String,
    pub rules: [String; 6], // raw text per tumbler, e.g. "3r, 6l" or "-"
    pub start: Option<[i32; 6]>,
    pub solution: Vec<String>, // grouped step lines, or a single note line
}

// ---------- rules -> delta matrix ----------

// D-press delta for one tumbler from its rule text. `idx` is the tumbler (0..5).
pub fn rule_to_delta(idx: usize, text: &str) -> Result<[i32; 6], String> {
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

pub fn build_matrix(rules: &[String; 6]) -> Result<[[i32; 6]; 6], String> {
    let mut m = [[0i32; 6]; 6];
    for i in 0..6 {
        m[i] = rule_to_delta(i, &rules[i])?;
    }
    Ok(m)
}

// ---------- solver ----------

pub type State = [i32; 6];

pub struct Solution {
    pub total: usize,
    pub steps: Vec<(usize, char, usize)>, // (tumbler 1..6, key, count)
}

pub fn solve(start: State, mat: &[[i32; 6]; 6]) -> Option<Solution> {
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

// Apply a single click to a state in place: [D] adds the tumbler's delta row,
// [A] subtracts it. Mirrors the transition used inside `solve`.
pub fn apply_click(state: &mut State, mat: &[[i32; 6]; 6], tumbler: usize, key: char) {
    let sgn = if key == 'A' { -1 } else { 1 };
    for i in 0..6 {
        state[i] += sgn * mat[tumbler][i];
    }
}

// ---------- history parsing ----------

pub fn parse_int_array(s: &str) -> Option<[i32; 6]> {
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

// Parse grouped solution lines like "4: 2x D" into (tumbler 1..6, key, count) —
// the same 1-based convention `solve` and `solution_lines` use.
// Returns None if any non-empty line doesn't match (e.g. a free-text note).
pub fn parse_solution_steps(lines: &[String]) -> Option<Vec<(usize, char, usize)>> {
    let mut out = Vec::new();
    for raw in lines {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let colon = line.find(':')?;
        let tumbler: usize = line[..colon].trim().parse().ok()?;
        if !(1..=6).contains(&tumbler) {
            return None;
        }
        let rest = line[colon + 1..].trim();
        // expected: "<count>x <key>"
        let xpos = rest.find('x')?;
        let count: usize = rest[..xpos].trim().parse().ok()?;
        let key = rest[xpos + 1..].trim().chars().next()?.to_ascii_uppercase();
        if key != 'A' && key != 'D' {
            return None;
        }
        out.push((tumbler, key, count));
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
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

pub fn parse_history(text: &str) -> Vec<Lock> {
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

// ---------- solve input parsing ----------

pub fn parse_input(text: &str) -> Result<(String, [String; 6], [i32; 6]), String> {
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

// ---------- output helpers ----------

pub fn solution_lines(sol: &Solution) -> Vec<String> {
    sol.steps
        .iter()
        .map(|(t, k, n)| format!("{}: {}x {}", t, n, k))
        .collect()
}

// Render a lock as the markdown section used in the history file.
pub fn lock_markdown(name: &str, rules: &[String; 6], start: &[i32; 6], sol: &Solution) -> String {
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
        "**Solution:** ({} clicks — solved by locks)\n```\n",
        sol.total
    ));
    for line in solution_lines(sol) {
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("```\n");
    out
}

// Append a solved lock to the history file. Returns an error string on I/O failure.
pub fn append_lock(
    file: &str,
    name: &str,
    rules: &[String; 6],
    start: &[i32; 6],
    sol: &Solution,
) -> Result<(), String> {
    let block = lock_markdown(name, rules, start, sol);
    let mut existing = fs::read_to_string(file).unwrap_or_default();
    existing.push_str(&block);
    fs::write(file, existing).map_err(|e| format!("cannot write {}: {}", file, e))
}

// Return `text` with the lock at `index` (0-based, in `parse_history` order)
// removed. A lock's section runs from its `## ` header to just before the next
// one (or end of file), so dropping that line range cleanly removes the entry.
// Returns None if `index` is out of range.
pub fn remove_lock(text: &str, index: usize) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.starts_with("## "))
        .map(|(i, _)| i)
        .collect();
    let start = *starts.get(index)?;
    let end = starts.get(index + 1).copied().unwrap_or(lines.len());

    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    out.extend_from_slice(&lines[..start]);
    out.extend_from_slice(&lines[end..]);
    // Trim any trailing blank lines left behind, then end with a single newline.
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }
    let mut s = out.join("\n");
    s.push('\n');
    Some(s)
}

// Remove the lock at `index` from the history file, rewriting it in place.
// Returns the removed lock's name on success.
pub fn remove_lock_from_file(file: &str, index: usize) -> Result<String, String> {
    let text = fs::read_to_string(file).map_err(|e| format!("cannot read {}: {}", file, e))?;
    let name = parse_history(&text)
        .get(index)
        .map(|l| l.name.clone())
        .ok_or_else(|| format!("no lock at position {}", index + 1))?;
    let updated =
        remove_lock(&text, index).ok_or_else(|| format!("no lock at position {}", index + 1))?;
    fs::write(file, updated).map_err(|e| format!("cannot write {}: {}", file, e))?;
    Ok(name)
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(arr: [&str; 6]) -> [String; 6] {
        arr.map(|s| s.to_string())
    }

    // Replay a grouped solution from `start`, returning the final state.
    // Errors if any plate leaves the 1..7 track mid-sequence (a wall hit).
    fn replay(
        start: State,
        mat: &[[i32; 6]; 6],
        steps: &[(usize, char, usize)],
    ) -> Result<State, String> {
        let mut s = start;
        for &(tumbler, key, count) in steps {
            for _ in 0..count {
                apply_click(&mut s, mat, tumbler - 1, key);
                for (i, &v) in s.iter().enumerate() {
                    if !(1..=7).contains(&v) {
                        return Err(format!("plate {} hit wall at {}", i + 1, v));
                    }
                }
            }
        }
        Ok(s)
    }

    #[test]
    fn rule_to_delta_basic() {
        // own plate moves -1; `r` is -1, `l` is +1 on the named plate.
        let d = rule_to_delta(0, "3r, 5l").unwrap();
        assert_eq!(d, [-1, 0, -1, 0, 1, 0]);
    }

    #[test]
    fn rule_to_delta_dash_is_self_only() {
        assert_eq!(rule_to_delta(2, "-").unwrap(), [0, 0, -1, 0, 0, 0]);
        assert_eq!(rule_to_delta(2, "").unwrap(), [0, 0, -1, 0, 0, 0]);
    }

    #[test]
    fn rule_to_delta_accumulates_repeats() {
        // same plate named twice should sum.
        let d = rule_to_delta(0, "2r, 2r").unwrap();
        assert_eq!(d[1], -2);
    }

    #[test]
    fn rule_to_delta_rejects_bad_input() {
        assert!(rule_to_delta(0, "9r").is_err()); // plate out of range
        assert!(rule_to_delta(0, "3x").is_err()); // bad direction
        assert!(rule_to_delta(0, "zr").is_err()); // non-numeric plate
    }

    #[test]
    fn apply_click_inverts() {
        let mat = build_matrix(&rules(["3r, 5l", "-", "-", "-", "-", "-"])).unwrap();
        let mut s = [4, 4, 4, 4, 4, 4];
        apply_click(&mut s, &mat, 0, 'D');
        apply_click(&mut s, &mat, 0, 'A');
        assert_eq!(s, [4, 4, 4, 4, 4, 4]); // D then A is a no-op
    }

    #[test]
    fn solve_already_at_goal() {
        let mat = build_matrix(&rules(["-"; 6])).unwrap();
        let sol = solve(GOAL, &mat).expect("goal is trivially solvable");
        assert_eq!(sol.total, 0);
        assert!(sol.steps.is_empty());
    }

    #[test]
    fn solve_independent_tumblers() {
        // every tumbler independent: one D each from position 5 reaches 4.
        let mat = build_matrix(&rules(["-"; 6])).unwrap();
        let sol = solve([5, 5, 5, 5, 5, 5], &mat).expect("solvable");
        assert_eq!(sol.total, 6);
        let end = replay([5, 5, 5, 5, 5, 5], &mat, &sol.steps).unwrap();
        assert_eq!(end, GOAL);
    }

    #[test]
    fn solve_detects_unsolvable_invariant() {
        // Tumblers 1 and 2 are chained so they always move together: their
        // difference is invariant, so a start with plate1 != plate2 can't centre both.
        let mat = build_matrix(&rules(["2r", "1r", "-", "-", "-", "-"])).unwrap();
        assert!(solve([4, 5, 4, 4, 4, 4], &mat).is_none());
    }

    #[test]
    fn solve_result_reaches_goal_within_walls() {
        // a non-trivial coupled lock (history lock 5)
        let r = rules(["3r, 6l", "-", "1r, 4l, 6r", "2r, 5r, 6l", "-", "3l"]);
        let mat = build_matrix(&r).unwrap();
        let start = [5, 3, 6, 7, 2, 7];
        let sol = solve(start, &mat).expect("solvable");
        let end = replay(start, &mat, &sol.steps).expect("no wall hit");
        assert_eq!(end, GOAL);
        // grouped steps must re-expand to exactly `total` clicks.
        let clicks: usize = sol.steps.iter().map(|(_, _, n)| n).sum();
        assert_eq!(clicks, sol.total);
    }

    #[test]
    fn parse_int_array_ok_and_bad() {
        assert_eq!(parse_int_array("[5, 3, 6, 7, 2, 7]"), Some([5, 3, 6, 7, 2, 7]));
        assert_eq!(parse_int_array("Start: `[1,4,5,2,1,4]`"), Some([1, 4, 5, 2, 1, 4]));
        assert_eq!(parse_int_array("[1, 2, 3]"), None); // wrong arity
        assert_eq!(parse_int_array("no brackets"), None);
    }

    #[test]
    fn parse_solution_steps_roundtrip() {
        let lines = vec!["4: 2x D".to_string(), "1: 3x A".to_string()];
        let steps = parse_solution_steps(&lines).unwrap();
        assert_eq!(steps, vec![(4, 'D', 2), (1, 'A', 3)]); // 1-based tumblers
    }

    #[test]
    fn parse_solution_steps_rejects_free_text() {
        assert!(parse_solution_steps(&["already centred".to_string()]).is_none());
        assert!(parse_solution_steps(&["7: 1x D".to_string()]).is_none()); // tumbler out of range
        assert!(parse_solution_steps(&[]).is_none());
    }

    #[test]
    fn solution_lines_format() {
        let sol = Solution {
            total: 3,
            steps: vec![(4, 'D', 2), (1, 'A', 1)],
        };
        assert_eq!(solution_lines(&sol), vec!["4: 2x D", "1: 1x A"]);
    }

    #[test]
    fn parse_input_reads_rules_and_start() {
        let text = "Name: test\nRules:\n1: 3r, 6l\n2: -\nStart:\n[5, 3, 6, 7, 2, 7]\n";
        let (name, rules, start) = parse_input(text).unwrap();
        assert_eq!(name, "test");
        assert_eq!(rules[0], "3r, 6l");
        assert_eq!(start, [5, 3, 6, 7, 2, 7]);
    }

    #[test]
    fn parse_input_requires_start() {
        let text = "Rules:\n1: 3r\n";
        assert!(parse_input(text).is_err());
    }

    #[test]
    fn parse_history_extracts_sections() {
        let md = "\
# History\n\n## My Lock\n\n**Rules:**\n```\n1: 2l\n2: -\n```\n\n\
**Start:** `[3, 4, 5, 6, 7, 1]`\n\n**Solution:**\n```\n1: 2x D\n2: 1x A\n```\n";
        let locks = parse_history(md);
        assert_eq!(locks.len(), 1);
        let l = &locks[0];
        assert_eq!(l.name, "My Lock");
        assert_eq!(l.rules[0], "2l");
        assert_eq!(l.start, Some([3, 4, 5, 6, 7, 1]));
        assert_eq!(l.solution, vec!["1: 2x D".to_string(), "2: 1x A".to_string()]);
    }

    #[test]
    fn remove_lock_drops_the_right_section() {
        let md = "\
# History\n\n## A\n\n**Start:** `[1, 1, 1, 1, 1, 1]`\n\n## B\n\nbody b\n\n## C\n\nbody c\n";
        // remove the middle lock
        let after = remove_lock(md, 1).unwrap();
        let names: Vec<String> = parse_history(&after).into_iter().map(|l| l.name).collect();
        assert_eq!(names, vec!["A".to_string(), "C".to_string()]);
        // remove the last lock
        let after_last = remove_lock(&after, 1).unwrap();
        let names: Vec<String> = parse_history(&after_last).into_iter().map(|l| l.name).collect();
        assert_eq!(names, vec!["A".to_string()]);
        // out of range
        assert!(remove_lock(md, 9).is_none());
    }

    #[test]
    fn lock_markdown_roundtrips_through_parser() {
        let r = rules(["3r, 6l", "-", "-", "-", "-", "-"]);
        let start = [5, 4, 4, 4, 4, 4];
        let sol = solve(start, &build_matrix(&r).unwrap()).unwrap();
        let md = lock_markdown("Roundtrip", &r, &start, &sol);
        let parsed = parse_history(&md);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "Roundtrip");
        assert_eq!(parsed[0].start, Some(start));
        assert_eq!(parsed[0].rules[0], "3r, 6l");
    }

    // Core-logic invariant: for every lock in the checked-in history that has a
    // start, the solver must find a solution whose OWN replay reaches the goal
    // without ever hitting a wall. This exercises solve + apply_click + parsing
    // end-to-end against real data, independent of the recorded solution string.
    #[test]
    fn solver_output_is_always_wall_safe() {
        let locks = parse_history(include_str!("../history-of-locks.md"));
        let mut checked = 0;
        for l in &locks {
            let start = match l.start {
                Some(s) => s,
                None => continue, // placeholder entries have no start
            };
            let mat = build_matrix(&l.rules)
                .unwrap_or_else(|e| panic!("lock '{}' has bad rules: {}", l.name, e));
            let sol = solve(start, &mat)
                .unwrap_or_else(|| panic!("lock '{}' is unsolvable per solver", l.name));
            let end = replay(start, &mat, &sol.steps)
                .unwrap_or_else(|e| panic!("lock '{}' solver output hit a wall: {}", l.name, e));
            assert_eq!(end, GOAL, "lock '{}' solver output misses the goal", l.name);
            checked += 1;
        }
        assert!(checked >= 5, "expected to check at least 5 locks with a start");
    }

    // Data-integrity check: the recorded grouped solution for every solved lock
    // must itself replay to the goal without a wall hit, and must be no shorter
    // than the optimal the solver finds. Guards against stale/mis-keyed entries.
    #[test]
    fn recorded_solutions_replay_to_goal() {
        let locks = parse_history(include_str!("../history-of-locks.md"));
        let mut checked = 0;
        for l in &locks {
            let (start, steps) = match (l.start, parse_solution_steps(&l.solution)) {
                (Some(s), Some(st)) => (s, st),
                _ => continue, // placeholder / note-only entries
            };
            let mat = build_matrix(&l.rules).unwrap();
            let end = replay(start, &mat, &steps).unwrap_or_else(|e| {
                panic!("lock '{}' recorded solution hit a wall: {}", l.name, e)
            });
            assert_eq!(end, GOAL, "lock '{}' recorded solution misses the goal", l.name);

            let recorded: usize = steps.iter().map(|(_, _, n)| n).sum();
            let optimal = solve(start, &mat).unwrap().total;
            assert!(
                optimal <= recorded,
                "lock '{}': solver found {} < recorded {} (recorded is non-optimal)",
                l.name,
                optimal,
                recorded
            );
            checked += 1;
        }
        assert!(checked >= 5, "expected to check at least 5 solved locks");
    }
}
