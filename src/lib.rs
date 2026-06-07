// lib.rs — core solver and history parsing for the Gothic Remake lock helper.
//
// Shared by the `locks` CLI (src/main.rs) and the TUI (src/tui.rs).
//
// A lock has N tumblers (2..8), each on a 1..7 track; the goal is every plate at
// 4. Pressing [D] on a tumbler moves its own plate -1, plus its rule effects. In
// a rule, `r` = that plate -1, `l` = that plate +1. [A] is the inverse of [D].
// The number of tumblers is inferred from the input (rule count / start length).

use std::collections::{HashMap, VecDeque};
use std::fs;

pub const DEFAULT_FILE: &str = "history-of-locks.md";
pub const MIN_TUMBLERS: usize = 2;
pub const MAX_TUMBLERS: usize = 8;

// The solved state for an N-tumbler lock: every plate centred on 4.
pub fn goal(n: usize) -> Vec<i32> {
    vec![4; n]
}

#[derive(Clone)]
pub struct Lock {
    pub name: String,
    pub rules: Vec<String>, // raw text per tumbler, e.g. "3r, 6l" or "-"
    pub start: Option<Vec<i32>>,
    pub solution: Vec<String>, // grouped step lines, or a single note line
}

// ---------- rules -> delta matrix ----------

// D-press delta for one tumbler from its rule text. `idx` is the tumbler (0..n).
pub fn rule_to_delta(idx: usize, text: &str, n: usize) -> Result<Vec<i32>, String> {
    let mut d = vec![0i32; n];
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
        if !(1..=n).contains(&num) {
            return Err(format!("plate {} out of range (1..{}) in rule {}", num, n, idx + 1));
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

// Build the NxN delta matrix from N rule strings (N = rules.len()).
pub fn build_matrix(rules: &[String]) -> Result<Vec<Vec<i32>>, String> {
    let n = rules.len();
    let mut m = Vec::with_capacity(n);
    for (i, r) in rules.iter().enumerate() {
        m.push(rule_to_delta(i, r, n)?);
    }
    Ok(m)
}

// ---------- solver ----------

pub type State = Vec<i32>;

pub struct Solution {
    pub total: usize,
    pub steps: Vec<(usize, char, usize)>, // (tumbler 1..n, key, count)
}

// BFS over all 7^N plate states. `start` and `mat` must agree on N.
pub fn solve(start: &[i32], mat: &[Vec<i32>]) -> Option<Solution> {
    let n = start.len();
    let goal = goal(n);
    let start = start.to_vec();
    let mut prev: HashMap<State, (State, usize, char)> = HashMap::new();
    let mut queue: VecDeque<State> = VecDeque::new();
    queue.push_back(start.clone());
    prev.insert(start.clone(), (start.clone(), usize::MAX, ' '));

    let mut reached = start == goal;
    while let Some(cur) = queue.pop_front() {
        if cur == goal {
            reached = true;
            break;
        }
        for t in 0..n {
            for (key, sgn) in [('D', 1i32), ('A', -1i32)] {
                let mut next = cur.clone();
                let mut ok = true;
                for i in 0..n {
                    next[i] += sgn * mat[t][i];
                    if next[i] < 1 || next[i] > 7 {
                        ok = false;
                        break;
                    }
                }
                if !ok || prev.contains_key(&next) {
                    continue;
                }
                prev.insert(next.clone(), (cur.clone(), t, key));
                queue.push_back(next);
            }
        }
    }
    if !reached {
        return None;
    }

    let mut path: Vec<(usize, char)> = Vec::new();
    let mut cur = goal;
    loop {
        let (p, t, key) = prev[&cur].clone();
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
// [A] subtracts it. `tumbler` is 1-based. Mirrors the transition in `solve`.
pub fn apply_click(state: &mut [i32], mat: &[Vec<i32>], tumbler: usize, key: char) {
    let sgn = if key == 'A' { -1 } else { 1 };
    for i in 0..state.len() {
        state[i] += sgn * mat[tumbler - 1][i];
    }
}

// ---------- history parsing ----------

// All integers inside the first [...] of `s`, in order (any count).
pub fn parse_int_array(s: &str) -> Option<Vec<i32>> {
    let l = s.find('[')?;
    let r = s.find(']')?;
    let parts: Vec<i32> = s[l + 1..r]
        .split(',')
        .filter_map(|x| x.trim().parse().ok())
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts)
    }
}

// Parse grouped solution lines like "4: 2x D" into (tumbler 1..n, key, count).
// Returns None if any non-empty line doesn't match (e.g. a free-text note).
pub fn parse_solution_steps(lines: &[String], n: usize) -> Option<Vec<(usize, char, usize)>> {
    let mut out = Vec::new();
    for raw in lines {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let colon = line.find(':')?;
        let tumbler: usize = line[..colon].trim().parse().ok()?;
        if !(1..=n).contains(&tumbler) {
            return None;
        }
        let rest = line[colon + 1..].trim();
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

// Read numbered "n: text" lines into a rules Vec sized so it also covers `start`.
fn rules_from_pairs(pairs: Vec<(usize, String)>, start_len: usize) -> Vec<String> {
    let max_idx = pairs.iter().map(|(i, _)| *i).max().unwrap_or(0);
    let n = max_idx.max(start_len);
    let mut rules = vec![String::new(); n];
    for (i, t) in pairs {
        if i >= 1 && i <= n {
            rules[i - 1] = t;
        }
    }
    rules
}

fn parse_section(name: String, body: &[&str]) -> Lock {
    let mut start = None;
    let mut solution = Vec::new();
    let mut rule_pairs: Vec<(usize, String)> = Vec::new();

    let find = |needle: &str| body.iter().position(|l| l.contains(needle));

    if let Some(si) = find("**Start") {
        start = parse_int_array(body[si]);
    }
    if let Some(ri) = find("**Rules") {
        if let Some(content) = fence_after(body, ri) {
            for line in content {
                let line = line.trim();
                if let Some(colon) = line.find(':') {
                    if let Ok(num) = line[..colon].trim().parse::<usize>() {
                        rule_pairs.push((num, line[colon + 1..].trim().to_string()));
                    }
                }
            }
        }
    }
    if let Some(soli) = find("**Solution") {
        if let Some(content) = fence_after(body, soli) {
            solution = content.into_iter().filter(|l| !l.trim().is_empty()).collect();
        } else if let Some(colon) = body[soli].find(':') {
            // inline note after the marker
            let note = body[soli][colon + 1..].trim().trim_start_matches("**").trim();
            if !note.is_empty() {
                solution.push(note.to_string());
            }
        }
    }

    let rules = rules_from_pairs(rule_pairs, start.as_ref().map_or(0, |s| s.len()));
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

// Parse a `solve` input block into (name, rules, start). The tumbler count N is
// inferred from the rule lines / start length and must be in MIN..=MAX_TUMBLERS,
// with a Start of exactly N positions.
pub fn parse_input(text: &str) -> Result<(String, Vec<String>, Vec<i32>), String> {
    let mut name = String::from("unnamed");
    let mut start: Option<Vec<i32>> = None;
    let mut rule_pairs: Vec<(usize, String)> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("name:") {
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
            if let Ok(num) = line[..colon].trim().parse::<usize>() {
                rule_pairs.push((num, line[colon + 1..].trim().to_string()));
            }
        }
    }
    let start = start.ok_or("input is missing a Start line like [5,3,6,7,2,7]")?;
    let rules = rules_from_pairs(rule_pairs, start.len());
    let n = rules.len();
    if !(MIN_TUMBLERS..=MAX_TUMBLERS).contains(&n) {
        return Err(format!(
            "lock has {} tumblers; supported range is {}..{}",
            n, MIN_TUMBLERS, MAX_TUMBLERS
        ));
    }
    if start.len() != n {
        return Err(format!(
            "Start has {} positions but the lock has {} tumblers",
            start.len(),
            n
        ));
    }
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
pub fn lock_markdown(name: &str, rules: &[String], start: &[i32], sol: &Solution) -> String {
    let mut out = String::new();
    out.push_str(&format!("\n## {}\n\n", name));
    out.push_str("**Rules:**\n```\n");
    for (i, r) in rules.iter().enumerate() {
        let r = if r.is_empty() { "-" } else { r };
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
    rules: &[String],
    start: &[i32],
    sol: &Solution,
) -> Result<(), String> {
    let block = lock_markdown(name, rules, start, sol);
    let mut existing = fs::read_to_string(file).unwrap_or_default();
    existing.push_str(&block);
    fs::write(file, existing).map_err(|e| format!("cannot write {}: {}", file, e))
}

// Return `text` with the lock at `index` (0-based, parse_history order) removed.
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
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }
    let mut s = out.join("\n");
    s.push('\n');
    Some(s)
}

// Return `text` with the lock at `index` replaced by a freshly rendered section
// (rules / start / solution). Returns None if `index` is out of range.
pub fn replace_lock(
    text: &str,
    index: usize,
    name: &str,
    rules: &[String],
    start: &[i32],
    sol: &Solution,
) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.starts_with("## "))
        .map(|(i, _)| i)
        .collect();
    let s = *starts.get(index)?;
    let end = starts.get(index + 1).copied().unwrap_or(lines.len());

    // everything before the lock, trailing blanks trimmed
    let mut out: Vec<String> = lines[..s].iter().map(|l| l.to_string()).collect();
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }
    if !out.is_empty() {
        out.push(String::new()); // one blank line before the section
    }

    // the new section (lock_markdown wraps with leading/trailing newlines)
    let section = lock_markdown(name, rules, start, sol);
    for l in section.trim_matches('\n').lines() {
        out.push(l.to_string());
    }

    // everything after the lock, with its leading blanks dropped
    let tail = &lines[end..];
    if tail.iter().any(|l| !l.trim().is_empty()) {
        out.push(String::new());
        let mut started = false;
        for l in tail {
            if !started && l.trim().is_empty() {
                continue;
            }
            started = true;
            out.push(l.to_string());
        }
    }
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }
    let mut joined = out.join("\n");
    joined.push('\n');
    Some(joined)
}

// Replace the lock at `index` in the history file, rewriting it in place.
pub fn replace_lock_in_file(
    file: &str,
    index: usize,
    name: &str,
    rules: &[String],
    start: &[i32],
    sol: &Solution,
) -> Result<(), String> {
    let text = fs::read_to_string(file).map_err(|e| format!("cannot read {}: {}", file, e))?;
    let updated = replace_lock(&text, index, name, rules, start, sol)
        .ok_or_else(|| format!("no lock at position {}", index + 1))?;
    fs::write(file, updated).map_err(|e| format!("cannot write {}: {}", file, e))
}

// Render a lock as a `solve`-style input block (for `locks edit`).
pub fn input_block(name: &str, rules: &[String], start: &[i32]) -> String {
    let mut out = format!("Name: {}\n\nRules:\n", name);
    for (i, r) in rules.iter().enumerate() {
        let r = if r.is_empty() { "-" } else { r };
        out.push_str(&format!("{}: {}\n", i + 1, r));
    }
    out.push_str(&format!(
        "\nStart:\n[{}]\n",
        start.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
    ));
    out
}

// Remove the lock at `index` from the history file, returning the removed name.
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

    fn rules(arr: &[&str]) -> Vec<String> {
        arr.iter().map(|s| s.to_string()).collect()
    }

    // Replay a grouped solution from `start`, returning the final state.
    // Errors if any plate leaves the 1..7 track mid-sequence (a wall hit).
    fn replay(
        start: &[i32],
        mat: &[Vec<i32>],
        steps: &[(usize, char, usize)],
    ) -> Result<Vec<i32>, String> {
        let mut s = start.to_vec();
        for &(tumbler, key, count) in steps {
            for _ in 0..count {
                apply_click(&mut s, mat, tumbler, key);
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
        let d = rule_to_delta(0, "3r, 5l", 6).unwrap();
        assert_eq!(d, vec![-1, 0, -1, 0, 1, 0]);
    }

    #[test]
    fn rule_to_delta_dash_is_self_only() {
        assert_eq!(rule_to_delta(2, "-", 6).unwrap(), vec![0, 0, -1, 0, 0, 0]);
        assert_eq!(rule_to_delta(2, "", 6).unwrap(), vec![0, 0, -1, 0, 0, 0]);
    }

    #[test]
    fn rule_to_delta_accumulates_repeats() {
        let d = rule_to_delta(0, "2r, 2r", 6).unwrap();
        assert_eq!(d[1], -2);
    }

    #[test]
    fn rule_to_delta_rejects_bad_input() {
        assert!(rule_to_delta(0, "9r", 6).is_err()); // plate out of range for n=6
        assert!(rule_to_delta(0, "3x", 6).is_err()); // bad direction
        assert!(rule_to_delta(0, "zr", 6).is_err()); // non-numeric plate
    }

    #[test]
    fn apply_click_inverts() {
        let mat = build_matrix(&rules(&["3r, 5l", "-", "-", "-", "-", "-"])).unwrap();
        let mut s = vec![4, 4, 4, 4, 4, 4];
        apply_click(&mut s, &mat, 1, 'D');
        apply_click(&mut s, &mat, 1, 'A');
        assert_eq!(s, vec![4, 4, 4, 4, 4, 4]);
    }

    #[test]
    fn solve_already_at_goal() {
        let mat = build_matrix(&rules(&["-"; 6])).unwrap();
        let sol = solve(&goal(6), &mat).expect("goal is trivially solvable");
        assert_eq!(sol.total, 0);
        assert!(sol.steps.is_empty());
    }

    #[test]
    fn solve_independent_tumblers() {
        let mat = build_matrix(&rules(&["-"; 6])).unwrap();
        let sol = solve(&[5, 5, 5, 5, 5, 5], &mat).expect("solvable");
        assert_eq!(sol.total, 6);
        let end = replay(&[5, 5, 5, 5, 5, 5], &mat, &sol.steps).unwrap();
        assert_eq!(end, goal(6));
    }

    #[test]
    fn solve_three_tumblers() {
        // a coupled 3-tumbler lock
        let mat = build_matrix(&rules(&["2r", "-", "1l"])).unwrap();
        let sol = solve(&[6, 2, 5], &mat).expect("solvable");
        let end = replay(&[6, 2, 5], &mat, &sol.steps).unwrap();
        assert_eq!(end, goal(3));
    }

    #[test]
    fn solve_eight_tumblers_independent() {
        let mat = build_matrix(&rules(&["-"; 8])).unwrap();
        let sol = solve(&vec![5; 8], &mat).expect("solvable");
        assert_eq!(sol.total, 8);
        assert_eq!(replay(&vec![5; 8], &mat, &sol.steps).unwrap(), goal(8));
    }

    #[test]
    fn solve_detects_unsolvable_invariant() {
        let mat = build_matrix(&rules(&["2r", "1r", "-", "-", "-", "-"])).unwrap();
        assert!(solve(&[4, 5, 4, 4, 4, 4], &mat).is_none());
    }

    #[test]
    fn solve_result_reaches_goal_within_walls() {
        let r = rules(&["3r, 6l", "-", "1r, 4l, 6r", "2r, 5r, 6l", "-", "3l"]);
        let mat = build_matrix(&r).unwrap();
        let start = [5, 3, 6, 7, 2, 7];
        let sol = solve(&start, &mat).expect("solvable");
        let end = replay(&start, &mat, &sol.steps).expect("no wall hit");
        assert_eq!(end, goal(6));
        let clicks: usize = sol.steps.iter().map(|(_, _, n)| n).sum();
        assert_eq!(clicks, sol.total);
    }

    #[test]
    fn parse_int_array_ok_and_bad() {
        assert_eq!(parse_int_array("[5, 3, 6, 7, 2, 7]"), Some(vec![5, 3, 6, 7, 2, 7]));
        assert_eq!(parse_int_array("Start: `[1,4,5]`"), Some(vec![1, 4, 5]));
        assert_eq!(parse_int_array("no brackets"), None);
    }

    #[test]
    fn parse_solution_steps_roundtrip() {
        let lines = vec!["4: 2x D".to_string(), "1: 3x A".to_string()];
        let steps = parse_solution_steps(&lines, 6).unwrap();
        assert_eq!(steps, vec![(4, 'D', 2), (1, 'A', 3)]);
    }

    #[test]
    fn parse_solution_steps_rejects_free_text() {
        assert!(parse_solution_steps(&["already centred".to_string()], 6).is_none());
        assert!(parse_solution_steps(&["7: 1x D".to_string()], 6).is_none()); // tumbler > n
        assert!(parse_solution_steps(&[], 6).is_none());
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
        let text = "Name: test\nRules:\n1: 3r, 6l\n2: -\n3: -\n4: -\n5: -\n6: -\nStart:\n[5, 3, 6, 7, 2, 7]\n";
        let (name, rules, start) = parse_input(text).unwrap();
        assert_eq!(name, "test");
        assert_eq!(rules.len(), 6);
        assert_eq!(rules[0], "3r, 6l");
        assert_eq!(start, vec![5, 3, 6, 7, 2, 7]);
    }

    #[test]
    fn parse_input_infers_three_tumblers() {
        let text = "Rules:\n1: 2r\n2: -\n3: 1l\nStart:\n[6, 2, 5]\n";
        let (_, rules, start) = parse_input(text).unwrap();
        assert_eq!(rules.len(), 3);
        assert_eq!(start.len(), 3);
    }

    #[test]
    fn parse_input_requires_start_and_valid_count() {
        assert!(parse_input("Rules:\n1: 3r\n").is_err()); // no start
        // 1 tumbler is below the supported range
        assert!(parse_input("Rules:\n1: -\nStart:\n[4]\n").is_err());
        // start count mismatch
        assert!(parse_input("Rules:\n1: -\n2: -\n3: -\nStart:\n[4, 4]\n").is_err());
    }

    #[test]
    fn parse_history_extracts_sections() {
        let md = "\
# History\n\n## My Lock\n\n**Rules:**\n```\n1: 2l\n2: -\n```\n\n\
**Start:** `[3, 4]`\n\n**Solution:**\n```\n1: 2x D\n2: 1x A\n```\n";
        let locks = parse_history(md);
        assert_eq!(locks.len(), 1);
        let l = &locks[0];
        assert_eq!(l.name, "My Lock");
        assert_eq!(l.rules.len(), 2);
        assert_eq!(l.rules[0], "2l");
        assert_eq!(l.start, Some(vec![3, 4]));
        assert_eq!(l.solution, vec!["1: 2x D".to_string(), "2: 1x A".to_string()]);
    }

    #[test]
    fn replace_lock_rewrites_in_place() {
        let md = "# History\n\n## A\n\nbody a\n\n## B\n\nold b\n\n## C\n\nbody c\n";
        let r = rules(&["3r", "-", "-"]);
        let sol = solve(&[5, 4, 4], &build_matrix(&r).unwrap()).unwrap();
        let after = replace_lock(md, 1, "B2", &r, &[5, 4, 4], &sol).unwrap();
        let locks = parse_history(&after);
        // same count, B replaced by B2 with the new rules/start
        assert_eq!(locks.iter().map(|l| l.name.clone()).collect::<Vec<_>>(), ["A", "B2", "C"]);
        assert_eq!(locks[1].rules, r);
        assert_eq!(locks[1].start, Some(vec![5, 4, 4]));
        assert_eq!(locks[0].name, "A"); // neighbours intact
        assert_eq!(locks[2].name, "C");
        assert!(replace_lock(md, 9, "x", &r, &[5, 4, 4], &sol).is_none());
    }

    #[test]
    fn remove_lock_drops_the_right_section() {
        let md = "# History\n\n## A\n\nbody a\n\n## B\n\nbody b\n\n## C\n\nbody c\n";
        let after = remove_lock(md, 1).unwrap();
        let names: Vec<String> = parse_history(&after).into_iter().map(|l| l.name).collect();
        assert_eq!(names, vec!["A".to_string(), "C".to_string()]);
        assert!(remove_lock(md, 9).is_none());
    }

    #[test]
    fn lock_markdown_roundtrips_through_parser() {
        let r = rules(&["3r, 6l", "-", "-", "-", "-", "-"]);
        let start = [5, 4, 4, 4, 4, 4];
        let sol = solve(&start, &build_matrix(&r).unwrap()).unwrap();
        let md = lock_markdown("Roundtrip", &r, &start, &sol);
        let parsed = parse_history(&md);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "Roundtrip");
        assert_eq!(parsed[0].start, Some(start.to_vec()));
        assert_eq!(parsed[0].rules.len(), 6);
        assert_eq!(parsed[0].rules[0], "3r, 6l");
    }

    // Core-logic invariant against the checked-in history.
    #[test]
    fn solver_output_is_always_wall_safe() {
        let locks = parse_history(include_str!("../history-of-locks.md"));
        let mut checked = 0;
        for l in &locks {
            let start = match &l.start {
                Some(s) => s.clone(),
                None => continue,
            };
            let mat = build_matrix(&l.rules)
                .unwrap_or_else(|e| panic!("lock '{}' has bad rules: {}", l.name, e));
            let sol = solve(&start, &mat)
                .unwrap_or_else(|| panic!("lock '{}' is unsolvable per solver", l.name));
            let end = replay(&start, &mat, &sol.steps)
                .unwrap_or_else(|e| panic!("lock '{}' solver output hit a wall: {}", l.name, e));
            assert_eq!(end, goal(start.len()), "lock '{}' misses the goal", l.name);
            checked += 1;
        }
        assert!(checked >= 5);
    }

    #[test]
    fn recorded_solutions_replay_to_goal() {
        let locks = parse_history(include_str!("../history-of-locks.md"));
        let mut checked = 0;
        for l in &locks {
            let (start, steps) = match (&l.start, parse_solution_steps(&l.solution, l.rules.len())) {
                (Some(s), Some(st)) => (s.clone(), st),
                _ => continue,
            };
            let mat = build_matrix(&l.rules).unwrap();
            let end = replay(&start, &mat, &steps)
                .unwrap_or_else(|e| panic!("lock '{}' recorded solution hit a wall: {}", l.name, e));
            assert_eq!(end, goal(start.len()), "lock '{}' recorded misses goal", l.name);

            let recorded: usize = steps.iter().map(|(_, _, n)| n).sum();
            let optimal = solve(&start, &mat).unwrap().total;
            assert!(optimal <= recorded, "lock '{}': {} > {}", l.name, optimal, recorded);
            checked += 1;
        }
        assert!(checked >= 5);
    }
}
