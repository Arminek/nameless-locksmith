// helpers.js — small presentation/parsing helpers for the web app.
//
// The actual solver (build_matrix + the BFS) runs in WebAssembly compiled from
// the shared Rust core (see web-wasm/). These are just trivial string/array
// utilities used by the UI; they intentionally contain no search logic.

// Apply one click in place. `tumbler` is 1-based; [A] is the inverse of [D].
export function applyClick(state, mat, tumbler, key) {
  const sgn = key === "A" ? -1 : 1;
  for (let i = 0; i < state.length; i++) state[i] += sgn * mat[tumbler - 1][i];
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
