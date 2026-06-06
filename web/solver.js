// solver.js — JavaScript port of the Rust core (src/lib.rs), generalized to an
// arbitrary number of tumblers N (2..8). Each tumbler slides on a 1..7 track and
// the goal is every plate at 4. The number of tumblers is inferred from the
// input (rules.length / start.length), so callers don't pass it explicitly.
//
// Conventions (plate frame): pressing [D] on a tumbler moves its own plate -1
// plus its rule effects; in a rule, `r` = that plate -1, `l` = that plate +1.
// [A] is the inverse of [D]. The BFS keeps the Rust iteration order (tumblers
// 0..N, keys D then A) so for 6-tumbler locks it returns identical solutions.

// Safety cap on explored states (7^8 ≈ 5.7M would be too heavy for a browser).
const MAX_STATES = 2_000_000;

export const goalFor = (n) => Array(n).fill(4);

// D-press delta for tumbler `idx` (0-based) of an N-tumbler lock. Returns {d}|{err}.
export function ruleToDelta(idx, text, n) {
  const d = Array(n).fill(0);
  d[idx] = -1; // own plate always moves -1 on a [D] press
  const t = (text ?? "").trim();
  if (t === "" || t === "-") return { d };
  for (let tok of t.split(",")) {
    tok = tok.trim();
    if (!tok) continue;
    const dir = tok[tok.length - 1];
    const num = Number(tok.slice(0, -1).trim());
    if (!Number.isInteger(num)) return { err: `bad token '${tok}' in rule ${idx + 1}` };
    if (num < 1 || num > n) return { err: `plate ${num} out of range (1..${n}) in rule ${idx + 1}` };
    if (dir === "r") d[num - 1] -= 1;
    else if (dir === "l") d[num - 1] += 1;
    else return { err: `unknown direction '${dir}' in rule ${idx + 1} (only r/l)` };
  }
  return { d };
}

// Build the NxN delta matrix from N rule strings. Returns {mat} or {err}.
export function buildMatrix(rules) {
  const n = rules.length;
  const mat = [];
  for (let i = 0; i < n; i++) {
    const { d, err } = ruleToDelta(i, rules[i], n);
    if (err) return { err };
    mat.push(d);
  }
  return { mat };
}

// Apply one click in place. `tumbler` is 1-based; [A] is the inverse of [D].
export function applyClick(state, mat, tumbler, key) {
  const sgn = key === "A" ? -1 : 1;
  for (let i = 0; i < state.length; i++) state[i] += sgn * mat[tumbler - 1][i];
}

const keyOf = (s) => s.join(",");

// BFS over all 7^N plate states. Returns {total, steps:[[tumbler1..N,key,count]]}
// for the shortest wall-safe sequence, null if unreachable, or {err} if the
// search exceeds the browser-safe state cap.
export function solve(start, mat) {
  const n = start.length;
  const goalKey = keyOf(goalFor(n));
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
    for (let t = 0; t < n; t++) {
      for (const [kc, sgn] of [["D", 1], ["A", -1]]) {
        const next = cur.slice();
        let ok = true;
        for (let i = 0; i < n; i++) {
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
    if (prev.size > MAX_STATES) return { err: "too-complex" };
  }
  if (!reached) return null;

  const path = [];
  let cur = goalFor(n);
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

// Parse grouped solution lines like "4: 2x D" into [[tumbler,key,count]].
// If `n` is given, tumblers must be within 1..n. Returns null on any mismatch.
export function parseSolutionSteps(lines, n) {
  const out = [];
  for (const raw of lines) {
    const line = raw.trim();
    if (!line) continue;
    const c = line.indexOf(":");
    if (c < 0) return null;
    const tumbler = Number(line.slice(0, c).trim());
    if (!Number.isInteger(tumbler) || tumbler < 1 || (n && tumbler > n)) return null;
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

// Parse integers from a free-form string (commas/spaces/brackets ok).
export function parseStart(s) {
  return (s.match(/-?\d+/g) ?? []).map(Number);
}

export const stepLine = ([t, k, n]) => `${t}: ${n}x ${k}`;
