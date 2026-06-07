// app.js — the web UI. Mirrors the TUI (Browse / Solve / Step) using the JS
// solver port and the shared i18n + history data.

import {
  buildMatrix,
  solve,
  applyClick,
  parseSolutionSteps,
  parseStart,
  stepLine,
} from "./solver.js";
import { LANGS, I18N, HISTORY } from "./data.js";

const SVGNS = "http://www.w3.org/2000/svg";
const $ = (s) => document.querySelector(s);
const el = (tag, props = {}, kids = []) => {
  const e = document.createElement(tag);
  Object.assign(e, props);
  for (const k of [].concat(kids)) e.append(k);
  return e;
};
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ---------- state ----------

const SAVE_KEY = "nl_saved_locks";
const loadSaved = () => {
  try {
    return JSON.parse(localStorage.getItem(SAVE_KEY) || "[]");
  } catch {
    return [];
  }
};
const state = {
  lang: localStorage.getItem("nl_lang"),
  screen: "lang",
  saved: loadSaved(),
  browse: { filter: "", sel: 0 },
  solve: { n: 6, name: "", rules: ["", "", "", "", "", ""], start: "", result: null },
  step: null,
};
if (state.lang && I18N[state.lang]) state.screen = "browse";

const tr = (key) => I18N[state.lang]?.[key] ?? I18N.en[key] ?? key;
const allLocks = () => [...HISTORY, ...state.saved];
const isSaved = (i) => i >= HISTORY.length;
const persist = () => localStorage.setItem(SAVE_KEY, JSON.stringify(state.saved));

// ---------- SVG lock visualization ----------

const SP = 40; // hole spacing
const CX = 230; // fixed pin column (screen x)
const HOLE0 = CX - 3 * SP; // x of hole 1 when the plate is centred (p=4)
const ROW_H = 40;
const TOP = 36;

function buildLock(n = 6) {
  const svg = document.createElementNS(SVGNS, "svg");
  svg.setAttribute("viewBox", `0 0 460 ${TOP + n * ROW_H + 12}`);
  svg.classList.add("locksvg");
  svg.style.width = "100%";

  // centre guide + goal marker
  const guide = document.createElementNS(SVGNS, "line");
  guide.setAttribute("x1", CX);
  guide.setAttribute("x2", CX);
  guide.setAttribute("y1", TOP - 8);
  guide.setAttribute("y2", TOP + n * ROW_H);
  guide.setAttribute("stroke", "#57c7d455");
  guide.setAttribute("stroke-dasharray", "3 4");
  svg.append(guide);
  const goal = document.createElementNS(SVGNS, "text");
  goal.setAttribute("x", CX);
  goal.setAttribute("y", TOP - 12);
  goal.setAttribute("fill", "#57c7d4");
  goal.setAttribute("text-anchor", "middle");
  goal.setAttribute("font-size", "13");
  goal.textContent = "▼ 4";
  svg.append(goal);

  const rows = [];
  for (let t = n - 1; t >= 0; t--) {
    const y = TOP + (n - 1 - t) * ROW_H + ROW_H / 2;
    const depth = t; // 0 = front
    const shade = 150 - depth * 14;
    const g = document.createElementNS(SVGNS, "g");
    g.classList.add("plate-row");

    // plate body
    const plate = document.createElementNS(SVGNS, "rect");
    plate.setAttribute("x", HOLE0 - 26);
    plate.setAttribute("y", y - 14);
    plate.setAttribute("width", 6 * SP + 52);
    plate.setAttribute("height", 28);
    plate.setAttribute("rx", 8);
    plate.setAttribute("fill", `rgb(${shade},${shade - 14},${shade - 40})`);
    plate.setAttribute("stroke", "#0006");
    g.append(plate);

    // holes 1..7
    for (let h = 1; h <= 7; h++) {
      const c = document.createElementNS(SVGNS, "circle");
      c.setAttribute("cx", HOLE0 + (h - 1) * SP);
      c.setAttribute("cy", y);
      c.setAttribute("r", h === 4 ? 8 : 6);
      c.setAttribute("fill", "#0e0b07");
      if (h === 4) {
        c.setAttribute("stroke", "#57c7d4aa");
        c.setAttribute("stroke-width", "2");
      }
      g.append(c);
    }
    svg.append(g);

    // label (left, fixed)
    const lbl = document.createElementNS(SVGNS, "text");
    lbl.setAttribute("x", 14);
    lbl.setAttribute("y", y + 4);
    lbl.setAttribute("fill", "#9a8e78");
    lbl.setAttribute("font-size", "13");
    lbl.textContent = t + 1;
    svg.append(lbl);

    // pin (fixed at CX) — brass body with a red head
    const pin = document.createElementNS(SVGNS, "g");
    pin.classList.add("pin");
    const body = document.createElementNS(SVGNS, "circle");
    body.setAttribute("cx", CX);
    body.setAttribute("cy", y);
    body.setAttribute("r", 7);
    body.setAttribute("fill", "#d9a441");
    const head = document.createElementNS(SVGNS, "circle");
    head.setAttribute("cx", CX);
    head.setAttribute("cy", y - 2);
    head.setAttribute("r", 3);
    head.setAttribute("fill", "#e0623e");
    pin.append(body, head);
    svg.append(pin);

    rows.push({ g, pin, body });
  }

  function update(pos, { fast = false, active = null } = {}) {
    rows.forEach((r, idx) => {
      const t = n - 1 - idx; // rows built top(n-1)..bottom(0); idx 0 => tumbler n
      const p = pos[t];
      r.g.classList.toggle("fast", fast);
      r.g.setAttribute("transform", `translate(${(4 - p) * SP},0)`);
      const seated = p === 4;
      r.pin.classList.toggle("seated", seated);
      r.body.setAttribute("fill", seated ? "#6fcf6f" : active === t + 1 ? "#ffcf6b" : "#d9a441");
    });
  }
  return { svg, update };
}

// ---------- views ----------

const view = $("#view");

function setStatus(s) {
  $("#status").textContent = s;
}

function render() {
  renderTabs();
  renderLangSelect();
  view.innerHTML = "";
  if (state.screen === "lang") return renderLangPick();
  if (state.screen === "browse") return renderBrowse();
  if (state.screen === "solve") return renderSolve();
  if (state.screen === "step") return renderStep();
}

function renderTabs() {
  const tabs = $("#tabs");
  tabs.innerHTML = "";
  if (state.screen === "lang") return;
  for (const [id, key] of [
    ["browse", "tab.browse"],
    ["solve", "tab.solve"],
    ["step", "tab.step"],
  ]) {
    const b = el("button", { textContent: tr(key) });
    if (state.screen === id) b.classList.add("active");
    b.onclick = () => {
      if (id === "step" && !state.step) return;
      state.screen = id;
      render();
    };
    tabs.append(b);
  }
}

function renderLangSelect() {
  $("#lang-label").textContent = state.lang ? "" : "Language";
  const sel = $("#lang-select");
  sel.innerHTML = "";
  for (const l of LANGS) sel.append(el("option", { value: l.code, textContent: l.name }));
  sel.value = state.lang || "en";
  sel.style.display = state.screen === "lang" ? "none" : "";
  sel.onchange = () => {
    state.lang = sel.value;
    localStorage.setItem("nl_lang", state.lang);
    render();
  };
}

function renderLangPick() {
  view.innerHTML = "";
  setStatus("");
  const grid = el("div", { className: "grid" });
  for (const l of LANGS) {
    grid.append(
      el("button", {
        textContent: l.name,
        onclick: () => {
          state.lang = l.code;
          localStorage.setItem("nl_lang", l.code);
          state.screen = "browse";
          render();
        },
      })
    );
  }
  view.append(
    el("div", { className: "langpick" }, [
      el("h1", { textContent: "Select language / Wybierz język" }),
      grid,
    ])
  );
}

function renderBrowse() {
  view.innerHTML = ""; // safe for direct re-renders (e.g. after delete)
  setStatus(`${tr("tab.browse")} · ${tr("tab.solve")} (Tab) · ${tr("step.open").replace(/✓ /, "")}`);
  const locks = allLocks();
  const q = state.browse.filter.toLowerCase();
  const shown = locks
    .map((l, i) => [l, i])
    .filter(([l]) => !q || l.name.toLowerCase().includes(q));
  if (state.browse.sel >= shown.length) state.browse.sel = Math.max(0, shown.length - 1);

  const filter = el("input", {
    className: "filter",
    placeholder: tr("filter.placeholder").replace("/ ", ""),
    value: state.browse.filter,
    oninput: (e) => {
      state.browse.filter = e.target.value;
      state.browse.sel = 0;
      renderBrowse.list();
    },
  });

  const list = el("ul", { className: "locklist" });
  const detail = el("div", { className: "kv" });

  const draw = () => {
    list.innerHTML = "";
    shown.forEach(([l, i], idx) => {
      const li = el("li", {}, [
        el("span", { className: "check", textContent: l.solution?.length ? "✓" : " " }),
        el("span", { textContent: l.name }),
      ]);
      if (idx === state.browse.sel) li.classList.add("sel");
      if (isSaved(i)) {
        const del = el("button", { className: "del", title: tr("browse.delete"), textContent: "🗑" });
        del.onclick = (e) => {
          e.stopPropagation();
          if (confirm(`${tr("browse.delete")} "${l.name}"?`)) {
            state.saved.splice(i - HISTORY.length, 1);
            persist();
            renderBrowse();
          }
        };
        li.append(del);
      }
      li.onclick = () => {
        state.browse.sel = idx;
        drawDetail();
        list.querySelectorAll("li").forEach((x, j) => x.classList.toggle("sel", j === idx));
      };
      li.ondblclick = () => openStepFromLock(l);
      list.append(li);
    });
    drawDetail();
  };
  const drawDetail = () => {
    detail.innerHTML = "";
    const entry = shown[state.browse.sel];
    if (!entry) {
      detail.append(el("div", { className: "muted", textContent: tr("detail.empty") }));
      return;
    }
    const l = entry[0];
    detail.append(el("div", { className: "mono", style: "color:var(--brass-bright)", textContent: l.name }));
    detail.append(
      el("div", { className: "muted", textContent: `${tr("label.rules")} · ${l.rules.length} ${tr("step.tumbler")}` })
    );
    for (let i = 0; i < l.rules.length; i++)
      detail.append(el("div", { className: "mono", textContent: `  ${i + 1}: ${l.rules[i] || "-"}` }));
    detail.append(
      el("div", {
        className: "mono muted",
        textContent: l.start ? `Start  [${l.start.join(", ")}]` : `Start  ${tr("value.none")}`,
      })
    );
    const canWalk = l.start && parseSolutionSteps(l.solution || [], l.rules.length);
    const walk = el("button", {
      className: "btn",
      textContent: `▶ ${tr("tab.step")}`,
      disabled: !canWalk,
      onclick: () => openStepFromLock(l),
    });
    detail.append(el("div", { style: "margin-top:10px" }, [walk]));
  };
  renderBrowse.list = draw;
  draw();

  view.append(
    el("div", { className: "wrap" }, [
      el("div", { className: "cols" }, [
        el("div", { className: "card" }, [
          el("h2", { textContent: `${tr("locks.title")} (${locks.length})` }),
          filter,
          list,
        ]),
        el("div", { className: "card" }, [el("h2", { textContent: tr("detail.title") }), detail]),
      ]),
    ])
  );
}

function openStepFromLock(l) {
  if (!l.start) return;
  const groups = parseSolutionSteps(l.solution || [], l.rules.length);
  if (!groups) return;
  const { mat, err } = buildMatrix(l.rules);
  if (err) return setStatus(err);
  startStep(l.name, mat, l.start.slice(), groups);
}

// ---------- solve ----------

// A worked example to load, so the format is concrete.
const EXAMPLE = {
  n: 6,
  name: "Second chest in the tower",
  rules: ["3r, 6l", "-", "1r, 4l, 6r", "2r, 5r, 6l", "-", "3l"],
  start: "5, 3, 6, 7, 2, 7",
};

// Resize the rules array to N, keeping existing entries.
function setTumblerCount(n) {
  const s = state.solve;
  s.n = n;
  const r = s.rules.slice(0, n);
  while (r.length < n) r.push("");
  s.rules = r;
}

function renderSolve() {
  view.innerHTML = ""; // safe for direct re-renders (tumbler count / example)
  setStatus(tr("status.solve").replace(/ · Esc.*/, ""));
  const s = state.solve;
  const form = el("div", { className: "solve-form" });
  const mkRow = (labelText, value, on, attrs = {}) => {
    const input = el("input", { value, oninput: (e) => on(e.target.value), ...attrs });
    return el("div", { className: "row" }, [el("label", { textContent: labelText }), input]);
  };

  // tumbler-count selector (2..8)
  const count = el("select");
  for (let i = 2; i <= 8; i++) count.append(el("option", { value: i, textContent: i, selected: i === s.n }));
  count.onchange = () => {
    setTumblerCount(Number(count.value));
    renderSolve();
  };
  form.append(el("div", { className: "row" }, [el("label", { textContent: tr("solve.tumblers") }), count]));

  form.append(mkRow(tr("field.name"), s.name, (v) => (s.name = v)));
  for (let i = 0; i < s.n; i++)
    form.append(mkRow(`${tr("field.rule")} ${i + 1}`, s.rules[i] ?? "", (v) => (s.rules[i] = v)));
  form.append(mkRow("Start", s.start, (v) => (s.start = v), { placeholder: Array(s.n).fill("4").join(", ") }));
  form.append(el("div", { className: "muted", textContent: tr("solve.hint") }));

  const solveBtn = el("button", { className: "btn", textContent: tr("solve.title"), onclick: runSolve });
  const exBtn = el("button", {
    className: "btn ghost",
    textContent: tr("solve.example"),
    onclick: () => {
      setTumblerCount(EXAMPLE.n);
      Object.assign(s, { name: EXAMPLE.name, rules: EXAMPLE.rules.slice(), start: EXAMPLE.start });
      renderSolve();
    },
  });
  form.append(el("div", { className: "btn-row" }, [solveBtn, exBtn]));
  form.append(formatHelp());

  const result = el("div", { className: "card", id: "solve-result" }, [
    el("h2", { textContent: tr("result.title") }),
    el("div", { id: "result-body" }, [el("div", { className: "muted", textContent: tr("result.empty") })]),
  ]);

  view.append(
    el("div", { className: "wrap" }, [
      el("div", { className: "cols" }, [
        el("div", { className: "card" }, [el("h2", { textContent: tr("solve.title") }), form]),
        result,
      ]),
    ])
  );
  if (s.result) showSolveResult(s.result);
}

// Collapsible format reference with a worked example (fully localized).
function formatHelp() {
  const d = el("details", { className: "help" });
  d.append(el("summary", { textContent: tr("solve.format") }));
  const body = el("div", { className: "help-body" });
  const p = (html) => {
    const e = el("p");
    e.innerHTML = html;
    return e;
  };
  body.append(
    p(tr("help.intro")),
    p(tr("help.rule")),
    p(tr("help.dirs")),
    p(tr("help.start")),
    p(`<b>${tr("help.example")}</b>`),
    el("pre", { className: "mono", textContent:
      "Rule 1: 3r, 6l\n" +
      "Rule 2: -\n" +
      "Rule 3: 1r, 4l, 6r\n" +
      "Rule 4: 2r, 5r, 6l\n" +
      "Rule 5: -\n" +
      "Rule 6: 3l\n" +
      "Start:  5, 3, 6, 7, 2, 7" }),
    p(`→ <b>${tr("help.solves")}</b>`),
    p(tr("help.output")),
    p(tr("help.tip"))
  );
  d.append(body);
  return d;
}

async function runSolve() {
  const s = state.solve;
  const start = parseStart(s.start);
  if (start.length !== s.n)
    return showResultMessage(`✗ ${tr("solve.err_count")} (${s.n})`, "error");
  const { mat, err } = buildMatrix(s.rules.slice(0, s.n));
  if (err) return showResultMessage(`✗ ${err}`, "error");
  const sol = solve(start, mat);
  if (sol && sol.err === "too-complex")
    return showResultMessage(`✗ ${tr("solve.err_complex")}`, "error");
  if (!sol) return showResultMessage(`✗ ${tr("solve.err_nosolution")}`, "error");

  s.result = { sol, mat, start, n: s.n, name: s.name.trim() || tr("value.unnamed") };
  await playCrack(s.n); // satisfying cracking animation
  showSolveResult(s.result);
}

// Spin the plates, then settle them onto hole 4 one at a time.
async function playCrack(n) {
  const body = $("#result-body");
  body.innerHTML = "";
  const lock = buildLock(n);
  body.append(
    el("div", { className: "muted", style: "text-align:center;margin-bottom:6px", textContent: `${tr("solve.cracking")} …` }),
    lock.svg
  );
  const pos = Array(n).fill(1);
  // spin phase
  for (let f = 0; f < 14; f++) {
    for (let t = 0; t < n; t++) pos[t] = ((f + t * 2) % 7) + 1;
    lock.update(pos, { fast: true });
    await sleep(55);
  }
  // cascade settle
  for (let t = 0; t < n; t++) {
    pos[t] = 4;
    lock.update(pos, {});
    await sleep(140);
  }
  await sleep(450);
}

function showResultMessage(msg, cls) {
  state.solve.result = null;
  const body = $("#result-body");
  if (!body) return;
  body.innerHTML = "";
  body.append(el("div", { className: cls || "", textContent: msg }));
}

function showSolveResult({ sol, mat, start, name, n }) {
  const body = $("#result-body");
  if (!body) return;
  body.innerHTML = "";
  body.append(
    el("div", { className: "ok", textContent: `✓ ${sol.total} ${tr("result.solved_suffix").replace(/ ·.*/, "")}` })
  );
  const walk = el("button", { className: "btn", textContent: `▶ ${tr("tab.step")}`, onclick: () => startStep(name, mat, start.slice(), sol.steps) });
  const save = el("button", {
    className: "btn ghost",
    textContent: `💾 ${tr("msg.saved")}`,
    onclick: () => {
      state.saved.push({ name, rules: state.solve.rules.slice(0, n), start: start.slice(), solution: sol.steps.map(stepLine) });
      persist();
      setStatus(`${tr("msg.saved")} "${name}"`);
    },
  });
  body.append(el("div", { className: "btn-row", style: "margin:8px 0" }, [walk, save]));
  const list = el("div", { className: "mono" });
  for (const st of sol.steps) list.append(el("div", { textContent: `  ${stepLine(st)}` }));
  body.append(list);
}

// ---------- step ----------

function startStep(name, mat, start, groups) {
  const clicks = [];
  for (const [t, k, n] of groups) for (let i = 0; i < n; i++) clicks.push([t, k]);
  state.step = { name, mat, start, groups, clicks, idx: 0 };
  state.screen = "step";
  render();
}

function stepPositions(st) {
  const s = st.start.slice();
  for (let i = 0; i < st.idx; i++) applyClick(s, st.mat, st.clicks[i][0], st.clicks[i][1]);
  return s;
}
function currentGroup(st) {
  if (st.idx >= st.clicks.length) return null;
  let acc = 0;
  for (let g = 0; g < st.groups.length; g++) {
    acc += st.groups[g][2];
    if (st.idx < acc) return g;
  }
  return null;
}

let autoTimer = null;
let stepGo = null; // smooth step navigation, set while the Step view is mounted

function renderStep() {
  if (autoTimer) {
    clearInterval(autoTimer);
    autoTimer = null;
  }
  const st = state.step;
  if (!st) {
    state.screen = "browse";
    return render();
  }
  view.innerHTML = "";
  setStatus(tr("status.step").replace(/ · Esc.*/, ""));
  const lock = buildLock(st.start.length);
  const moveBox = el("div", { className: "move" });
  const stepsList = el("ul", { className: "steps" });
  const progress = el("div", { className: "progress" });

  const update = () => {
    const pos = stepPositions(st);
    const cur = currentGroup(st);
    const active = cur === null ? null : st.groups[cur][0];
    lock.update(pos, { active });

    // current move keycap
    moveBox.innerHTML = "";
    const solved = pos.every((p) => p === 4) && st.idx === st.clicks.length;
    if (solved) {
      moveBox.append(el("div", { className: "ok", style: "font-size:20px", textContent: tr("step.open") }));
    } else if (cur !== null) {
      const [t, k, n] = st.groups[cur];
      const arrow = k === "D" ? "→" : "←";
      moveBox.append(
        el("div", { className: "who", textContent: `${tr("step.tumbler")} ${t}` }),
        el("div", { className: "keycap", textContent: k }),
        el("div", { className: "muted", textContent: `× ${n}` }),
        el("div", { className: "arrows", textContent: Array(n).fill(arrow).join(" ") })
      );
    }

    // steps checklist
    stepsList.innerHTML = "";
    st.groups.forEach(([t, k, n], g) => {
      const li = el("li", {}, [
        el("span", { className: "mark", textContent: g === cur ? "▶" : cur === null || g < cur ? "✓" : "" }),
        el("span", { textContent: `${t}   ${n}× ${k}` }),
      ]);
      if (g === cur) li.classList.add("cur");
      else if (cur === null || g < cur) li.classList.add("done");
      stepsList.append(li);
    });
    const selG = cur ?? st.groups.length - 1;
    stepsList.children[selG]?.scrollIntoView({ block: "nearest" });

    const centered = pos.filter((p) => p === 4).length;
    progress.textContent = solved
      ? ""
      : `${tr("step.click")} ${st.idx} / ${st.clicks.length}  ·  ${centered}/${st.start.length} ${tr("step.pins")}`;
  };

  const go = (d) => {
    st.idx = Math.max(0, Math.min(st.clicks.length, st.idx + d));
    update();
  };
  stepGo = go;
  const auto = el("button", { className: "btn", textContent: "▶︎ auto" });
  auto.onclick = () => {
    if (autoTimer) {
      clearInterval(autoTimer);
      autoTimer = null;
      auto.textContent = "▶︎ auto";
      return;
    }
    auto.textContent = "⏸ auto";
    autoTimer = setInterval(() => {
      if (st.idx >= st.clicks.length) {
        clearInterval(autoTimer);
        autoTimer = null;
        auto.textContent = "▶︎ auto";
        return;
      }
      go(1);
    }, 360);
  };

  const controls = el("div", { className: "controls" }, [
    el("button", { className: "btn ghost", textContent: "⏮", onclick: () => go(-st.clicks.length) }),
    el("button", { className: "btn ghost", textContent: "‹ prev", onclick: () => go(-1) }),
    el("button", { className: "btn", textContent: "next ›", onclick: () => go(1) }),
    auto,
    el("span", { className: "spacer" }),
    el("button", { className: "btn ghost", textContent: "⏭", onclick: () => go(st.clicks.length) }),
  ]);

  view.append(
    el("div", { className: "wrap" }, [
      el("div", { className: "step-grid" }, [
        el("div", { className: "card lockwrap" }, [
          el("h2", { textContent: `${tr("step.lock")} — ${st.name}` }),
          el("div", { className: "muted", style: "margin-bottom:6px", textContent: tr("step.goal") }),
          lock.svg,
          controls,
          progress,
        ]),
        el("div", { className: "card" }, [
          el("h2", { textContent: tr("step.current") }),
          moveBox,
          el("h2", { style: "margin-top:14px", textContent: tr("step.steps") }),
          stepsList,
        ]),
      ]),
    ])
  );
  update();
}

// ---------- keyboard ----------

document.addEventListener("keydown", (e) => {
  if (state.screen === "step" && stepGo) {
    if (e.key === "ArrowRight" || e.key === " ") {
      e.preventDefault();
      stepGo(1);
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      stepGo(-1);
    }
  }
});

render();
