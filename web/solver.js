// solver.js — JavaScript port of the Rust core (src/lib.rs).
//
// Conventions (plate frame): track positions 1..7, goal = every plate at 4.
//   Pressing [D] on a tumbler moves its own plate -1, plus its rule effects.
//   In a rule, `r` = that plate -1, `l` = that plate +1. [A] is the inverse of [D].
//
// Kept deliberately faithful to the Rust BFS (same iteration order: tumblers
// 0..6, keys D then A) so it returns the same optimal solutions. Verified
// against the Rust binary by web/verify.mjs.

export const GOAL = [4, 4, 4, 4, 4, 4];

// D-press delta for one tumbler (0..5) from its rule text. Returns {d} or {err}.
export function ruleToDelta(idx, text) {
  const d = [0, 0, 0, 0, 0, 0];
  d[idx] = -1; // own plate always moves -1 on a [D] press
  const t = (text ?? "").trim();
  if (t === "" || t === "-") return { d };
  for (let tok of t.split(",")) {
    tok = tok.trim();
    if (!tok) continue;
    const dir = tok[tok.length - 1];
    const num = Number(tok.slice(0, -1).trim());
    if (!Number.isInteger(num)) return { err: `bad token '${tok}' in rule ${idx + 1}` };
    if (num < 1 || num > 6) return { err: `plate ${num} out of range in rule ${idx + 1}` };
    if (dir === "r") d[num - 1] -= 1;
    else if (dir === "l") d[num - 1] += 1;
    else return { err: `unknown direction '${dir}' in rule ${idx + 1} (only r/l)` };
  }
  return { d };
}

// Build the 6x6 delta matrix from six rule strings. Returns {mat} or {err}.
export function buildMatrix(rules) {
  const mat = [];
  for (let i = 0; i < 6; i++) {
    const { d, err } = ruleToDelta(i, rules[i]);
    if (err) return { err };
    mat.push(d);
  }
  return { mat };
}

// Apply one click in place. `tumbler` is 1-based; [A] is the inverse of [D].
export function applyClick(state, mat, tumbler, key) {
  const sgn = key === "A" ? -1 : 1;
  for (let i = 0; i < 6; i++) state[i] += sgn * mat[tumbler - 1][i];
}

const keyOf = (s) => s.join(",");

// BFS over all 7^6 plate states. Returns {total, steps:[[tumbler1..6,key,count]]}
// for the shortest wall-safe sequence, or null if the goal is unreachable.
export function solve(start, mat) {
  const goalKey = keyOf(GOAL);
  const prev = new Map();
  const queue = [start.slice()];
  prev.set(keyOf(start), { parent: null, t: -1, k: " " });

  let reached = keyOf(start) === goalKey;
  for (let head = 0; head < queue.length; head++) {
    const cur = queue[head];
    if (keyOf(cur) === goalKey) {
      reached = true;
      break;
    }
    for (let t = 0; t < 6; t++) {
      for (const [kc, sgn] of [["D", 1], ["A", -1]]) {
        const next = cur.slice();
        let ok = true;
        for (let i = 0; i < 6; i++) {
          next[i] += sgn * mat[t][i];
          if (next[i] < 1 || next[i] > 7) {
            ok = false;
            break;
          }
        }
        if (!ok) continue;
        const nk = keyOf(next);
        if (prev.has(nk)) continue;
        prev.set(nk, { parent: cur, t, k: kc });
        queue.push(next);
      }
    }
  }
  if (!reached) return null;

  const path = [];
  let cur = GOAL.slice();
  for (;;) {
    const info = prev.get(keyOf(cur));
    if (info.t === -1) break;
    path.push([info.t + 1, info.k]);
    cur = info.parent;
  }
  path.reverse();

  const steps = [];
  for (const [t, k] of path) {
    const last = steps[steps.length - 1];
    if (last && last[0] === t && last[1] === k) last[2] += 1;
    else steps.push([t, k, 1]);
  }
  return { total: path.length, steps };
}

// Parse grouped solution lines like "4: 2x D" into [[tumbler1..6,key,count]].
// Returns null if any non-empty line doesn't match.
export function parseSolutionSteps(lines) {
  const out = [];
  for (const raw of lines) {
    const line = raw.trim();
    if (!line) continue;
    const c = line.indexOf(":");
    if (c < 0) return null;
    const tumbler = Number(line.slice(0, c).trim());
    if (!Number.isInteger(tumbler) || tumbler < 1 || tumbler > 6) return null;
    const rest = line.slice(c + 1).trim();
    const x = rest.indexOf("x");
    if (x < 0) return null;
    const count = Number(rest.slice(0, x).trim());
    if (!Number.isInteger(count)) return null;
    const key = (rest.slice(x + 1).trim()[0] ?? "").toUpperCase();
    if (key !== "A" && key !== "D") return null;
    out.push([tumbler, key, count]);
  }
  return out.length ? out : null;
}

// Parse 6 integers from a free-form string (commas/spaces/brackets ok).
export function parseStart(s) {
  const nums = (s.match(/-?\d+/g) ?? []).map(Number);
  return nums.length === 6 ? nums : null;
}

export const stepLine = ([t, k, n]) => `${t}: ${n}x ${k}`;
