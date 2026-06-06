// Verifies the JS solver against the checked-in history: every lock must solve
// to the goal without hitting a wall, the recorded solution must replay to the
// goal, and the solver's optimal length must be <= the recorded one.
//
//   node web/verify.mjs   (exits non-zero on any failure; run in CI before deploy)

import { goalFor, buildMatrix, solve, applyClick, parseSolutionSteps } from "./solver.js";
import { HISTORY } from "./data.js";

const eq = (a, b) => a.length === b.length && a.every((v, i) => v === b[i]);

function replay(start, mat, steps) {
  const s = start.slice();
  for (const [t, k, n] of steps) {
    for (let i = 0; i < n; i++) {
      applyClick(s, mat, t, k);
      for (let j = 0; j < 6; j++) {
        if (s[j] < 1 || s[j] > 7) throw new Error(`plate ${j + 1} hit wall at ${s[j]}`);
      }
    }
  }
  return s;
}

let checked = 0;
let failed = 0;
for (const lock of HISTORY) {
  if (!lock.start) continue;
  const { mat, err } = buildMatrix(lock.rules);
  if (err) {
    console.error(`✗ ${lock.name}: ${err}`);
    failed++;
    continue;
  }
  const GOAL = goalFor(lock.start.length);
  try {
    // 1. solver output is wall-safe and reaches the goal
    const sol = solve(lock.start, mat);
    if (!sol || sol.err) throw new Error("solver found no solution");
    const end = replay(lock.start, mat, sol.steps);
    if (!eq(end, GOAL)) throw new Error(`solver output ends at [${end}]`);

    // 2. recorded solution replays to the goal and isn't shorter than optimal
    const recorded = parseSolutionSteps(lock.solution, lock.rules.length);
    if (recorded) {
      const rend = replay(lock.start, mat, recorded);
      if (!eq(rend, GOAL)) throw new Error(`recorded solution ends at [${rend}]`);
      const recordedClicks = recorded.reduce((a, [, , n]) => a + n, 0);
      if (sol.total > recordedClicks) {
        throw new Error(`solver ${sol.total} > recorded ${recordedClicks}`);
      }
      console.log(`✓ ${lock.name} — ${sol.total} clicks (recorded ${recordedClicks})`);
    } else {
      console.log(`✓ ${lock.name} — ${sol.total} clicks`);
    }
    checked++;
  } catch (e) {
    console.error(`✗ ${lock.name}: ${e.message}`);
    failed++;
  }
}

console.log(`\n${checked} locks verified, ${failed} failed`);
if (failed > 0 || checked < 5) process.exit(1);
