// WebAssembly bindings for the nameless-locksmith solver. This wraps the SAME
// core crate the CLI and TUI use, so the web app runs the real Rust solver —
// no separate JavaScript port to keep in sync.

use serde::Serialize;
use wasm_bindgen::prelude::*;

use nameless_locksmith::{build_matrix as core_build_matrix, solve as core_solve};

#[derive(Serialize)]
struct WasmSolution {
    total: usize,
    // [tumbler (1-based), key ("A"/"D"), count]
    steps: Vec<(usize, String, usize)>,
}

// Build the NxN delta matrix from N rule strings. Throws on a bad rule.
#[wasm_bindgen]
pub fn build_matrix(rules: Vec<String>) -> Result<JsValue, JsValue> {
    let mat = core_build_matrix(&rules).map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&mat).map_err(|e| JsValue::from_str(&e.to_string()))
}

// Solve a lock. Returns { total, steps } on success, `null` if the goal is
// unreachable without hitting a wall, or throws on a bad rule / size mismatch.
#[wasm_bindgen]
pub fn solve_lock(rules: Vec<String>, start: Vec<i32>) -> Result<JsValue, JsValue> {
    if start.len() != rules.len() {
        return Err(JsValue::from_str("start length must equal the number of rules"));
    }
    let mat = core_build_matrix(&rules).map_err(|e| JsValue::from_str(&e))?;
    match core_solve(&start, &mat) {
        None => Ok(JsValue::NULL),
        Some(sol) => {
            let steps = sol
                .steps
                .iter()
                .map(|(t, k, n)| (*t, k.to_string(), *n))
                .collect();
            let out = WasmSolution {
                total: sol.total,
                steps,
            };
            serde_wasm_bindgen::to_value(&out).map_err(|e| JsValue::from_str(&e.to_string()))
        }
    }
}
