// tui.rs — interactive terminal UI for the `locks` helper (ratatui + crossterm).
//
// Launched when `locks` is run with no subcommand. Screens:
//   Language — pick the UI language on startup.
//   Browse   — filterable history list + detail pane; Enter walks a saved solution.
//   Solve    — in-place form (name + 6 rules + start); runs BFS, shows the result.
//   Step     — walk a solution click-by-click. The plates slide past a fixed pin,
//              mirroring the in-game animation; tumbler 1 sits in the foreground.
//
// All solver/parsing logic is shared with the CLI via the library crate.
// UI strings live in src/i18n/<code>.txt — see the localization section below.

use std::collections::HashMap;
use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph, Wrap},
    DefaultTerminal, Frame,
};

use nameless_locksmith::{
    append_lock, build_matrix, parse_history, parse_solution_steps, remove_lock_from_file,
    replace_lock_in_file, solve, Lock, Solution, MAX_TUMBLERS, MIN_TUMBLERS,
};

// ---------- public entry point ----------

pub fn run(file: &str) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new(file);
    let res = app.run(&mut terminal);
    ratatui::restore();
    res
}

// ---------- localization ----------
//
// Each language is a `key = value` file embedded at compile time. To add a
// language: drop src/i18n/<code>.txt and add one row here. Missing keys fall
// back to the first entry (English), so partial translations still work.
const LANGUAGES: &[(&str, &str, &str)] = &[
    ("en", "English", include_str!("i18n/en.txt")),
    ("pl", "Polski", include_str!("i18n/pl.txt")),
    ("de", "Deutsch", include_str!("i18n/de.txt")),
    ("ru", "Русский", include_str!("i18n/ru.txt")),
    ("uk", "Українська", include_str!("i18n/uk.txt")),
    ("es", "Español", include_str!("i18n/es.txt")),
    ("pt", "Português", include_str!("i18n/pt.txt")),
    ("fr", "Français", include_str!("i18n/fr.txt")),
];

fn parse_catalog(src: &'static str) -> HashMap<&'static str, &'static str> {
    src.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (k, v) = line.split_once('=')?;
            Some((k.trim(), v.trim()))
        })
        .collect()
}

struct I18n {
    map: HashMap<&'static str, &'static str>,
    fallback: HashMap<&'static str, &'static str>, // English, for missing keys
}

impl I18n {
    fn new(idx: usize) -> Self {
        let idx = idx.min(LANGUAGES.len() - 1);
        I18n {
            map: parse_catalog(LANGUAGES[idx].2),
            fallback: parse_catalog(LANGUAGES[0].2),
        }
    }

    // Look up a key; falls back to English, then to the key itself if unknown.
    fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.map
            .get(key)
            .or_else(|| self.fallback.get(key))
            .copied()
            .unwrap_or(key)
    }
}

// ---------- app state ----------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Screen {
    Language,
    Browse,
    Solve,
    Step,
}

struct App {
    file: String,
    locks: Vec<Lock>,
    i18n: I18n,
    lang_sel: usize, // highlighted row on the Language screen / active language index
    screen: Screen,
    browse: BrowseState,
    solve: SolveState,
    step: Option<StepState>,
    step_list: ListState, // scroll state for the Steps pane
    step_origin: Screen,
    solve_anim: Option<u32>, // frame counter for the "cracking" animation after a solve
    status: String,
    quit: bool,
}

// Cracking-animation timing, in frames (~45 ms each).
const ANIM_SPIN: u32 = 7; // initial all-spin frames before plates start settling
const ANIM_LOCK_STEP: u32 = 3; // frames between each tumbler locking onto hole 4
const ANIM_HOLD: u32 = 7; // frames to hold the open lock before revealing the steps
const ANIM_END: u32 = ANIM_SPIN + 8 * ANIM_LOCK_STEP + ANIM_HOLD;

// Plate positions mid-animation: each tumbler spins through positions, then
// snaps to hole 4 in a cascade (tumbler 1 first), like a settling combination.
fn anim_positions(frame: u32, n: usize) -> Vec<i32> {
    (0..n)
        .map(|t| {
            let lock_at = ANIM_SPIN + t as u32 * ANIM_LOCK_STEP;
            if frame >= lock_at {
                4
            } else {
                ((frame + t as u32 * 2) % 7) as i32 + 1 // spinning reel
            }
        })
        .collect()
}

struct BrowseState {
    filter: String,
    filtering: bool,      // editing the filter box
    confirm_delete: bool, // awaiting y/n confirmation to delete the selected lock
    selected: usize,      // index into the filtered list
    list_state: ListState,
}

struct SolveState {
    n: usize, // tumbler count (2..8)
    name: String,
    rules: Vec<String>, // length n
    start: String,
    // focus fields: 0 = tumbler count, 1 = name, 2..=n+1 = rules, n+2 = start
    focus: usize,
    result: SolveResult,
    editing: Option<usize>, // history index being edited (^S replaces it), else None
}

impl SolveState {
    fn field_count(&self) -> usize {
        self.n + 3
    }
    // Resize the rules to the tumbler count, keeping existing entries.
    fn set_count(&mut self, n: usize) {
        self.n = n.clamp(MIN_TUMBLERS, MAX_TUMBLERS);
        self.rules.resize(self.n, String::new());
        if self.focus >= self.field_count() {
            self.focus = self.field_count() - 1;
        }
    }
}

enum SolveResult {
    None,
    Error(String),
    Solved {
        total: usize,
        lines: Vec<String>,
        mat: Vec<Vec<i32>>,
        start: Vec<i32>,
        groups: Vec<(usize, char, usize)>,
    },
}

// A solution being walked one click at a time.
struct StepState {
    name: String,
    mat: Vec<Vec<i32>>,
    start: Vec<i32>,
    clicks: Vec<(usize, char)>,        // expanded per-click: (tumbler 1..n, key)
    groups: Vec<(usize, char, usize)>, // grouped 1-based, for display
    idx: usize,                        // clicks applied so far (0..=clicks.len())
}

impl StepState {
    fn new(
        name: String,
        mat: Vec<Vec<i32>>,
        start: Vec<i32>,
        groups: Vec<(usize, char, usize)>,
    ) -> Self {
        let mut clicks = Vec::new();
        for &(t, k, n) in &groups {
            for _ in 0..n {
                clicks.push((t, k));
            }
        }
        StepState {
            name,
            mat,
            start,
            clicks,
            groups,
            idx: 0,
        }
    }

    // Plate positions after applying the first `idx` clicks.
    fn positions(&self) -> Vec<i32> {
        let mut s = self.start.clone();
        for &(t, k) in &self.clicks[..self.idx] {
            let sgn = if k == 'A' { -1 } else { 1 };
            for i in 0..s.len() {
                s[i] += sgn * self.mat[t - 1][i]; // tumblers are 1-based
            }
        }
        s
    }

    // Index of the group the *next* click belongs to (None when complete).
    fn current_group(&self) -> Option<usize> {
        if self.idx >= self.clicks.len() {
            return None;
        }
        let mut acc = 0;
        for (g, &(_, _, n)) in self.groups.iter().enumerate() {
            acc += n;
            if self.idx < acc {
                return Some(g);
            }
        }
        None
    }
}

impl App {
    fn new(file: &str) -> Self {
        let locks = load_locks(file);
        let mut list_state = ListState::default();
        if !locks.is_empty() {
            list_state.select(Some(0));
        }
        App {
            file: file.to_string(),
            locks,
            i18n: I18n::new(0),
            lang_sel: 0,
            screen: Screen::Language,
            browse: BrowseState {
                filter: String::new(),
                filtering: false,
                confirm_delete: false,
                selected: 0,
                list_state,
            },
            solve: SolveState {
                n: 6,
                name: String::new(),
                rules: vec![String::new(); 6],
                start: String::new(),
                focus: 1, // start on the name field
                result: SolveResult::None,
                editing: None,
            },
            step: None,
            step_list: ListState::default(),
            step_origin: Screen::Browse,
            solve_anim: None,
            status: String::new(),
            quit: false,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.quit {
            terminal.draw(|f| self.draw(f))?;
            if self.solve_anim.is_some() {
                // Animating: redraw on a timer, but still react to keys instantly.
                if event::poll(Duration::from_millis(45))? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            self.on_key(key);
                        }
                    }
                } else {
                    self.tick_anim();
                }
            } else if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.on_key(key);
                }
            }
        }
        Ok(())
    }

    fn tick_anim(&mut self) {
        self.solve_anim = match self.solve_anim {
            Some(f) if f < ANIM_END => Some(f + 1),
            _ => None,
        };
    }

    // Shorthand for a localized string by key.
    fn tr<'a>(&'a self, key: &'a str) -> &'a str {
        self.i18n.get(key)
    }

    // ---------- input ----------

    fn on_key(&mut self, key: KeyEvent) {
        match self.screen {
            Screen::Language => self.on_key_language(key),
            Screen::Browse => self.on_key_browse(key),
            Screen::Solve => self.on_key_solve(key),
            Screen::Step => self.on_key_step(key),
        }
    }

    fn on_key_language(&mut self, key: KeyEvent) {
        let n = LANGUAGES.len();
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Down | KeyCode::Char('j') => self.lang_sel = (self.lang_sel + 1) % n,
            KeyCode::Up | KeyCode::Char('k') => self.lang_sel = (self.lang_sel + n - 1) % n,
            KeyCode::Enter => self.confirm_language(),
            // first letter of a language code (e -> en, p -> pl) jumps and confirms
            KeyCode::Char(c) if c.is_alphabetic() => {
                let c = c.to_ascii_lowercase();
                if let Some(i) = LANGUAGES.iter().position(|l| l.0.starts_with(c)) {
                    self.lang_sel = i;
                    self.confirm_language();
                }
            }
            _ => {}
        }
    }

    fn confirm_language(&mut self) {
        self.i18n = I18n::new(self.lang_sel);
        self.screen = Screen::Browse;
        self.status = self.tr("status.browse").to_string();
    }

    fn on_key_browse(&mut self, key: KeyEvent) {
        if self.browse.confirm_delete {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => self.delete_selected(),
                _ => {
                    self.browse.confirm_delete = false;
                    self.status = self.tr("status.browse").to_string();
                }
            }
            return;
        }
        if self.browse.filtering {
            match key.code {
                KeyCode::Esc => {
                    self.browse.filter.clear();
                    self.browse.filtering = false;
                }
                KeyCode::Enter => self.browse.filtering = false,
                KeyCode::Backspace => {
                    self.browse.filter.pop();
                    self.clamp_browse_selection();
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.browse.filter.push(c);
                    self.clamp_browse_selection();
                }
                _ => {}
            }
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Tab => {
                self.solve.editing = None; // Tab = compose a new lock
                self.screen = Screen::Solve;
                self.status = self.tr("status.solve").to_string();
            }
            KeyCode::Char('/') => {
                self.browse.filtering = true;
                self.status = self.tr("status.filter").to_string();
            }
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Char('d') | KeyCode::Delete => self.request_delete(),
            KeyCode::Char('e') => self.start_edit(),
            KeyCode::Enter => self.enter_step_from_browse(),
            _ => {}
        }
    }

    // Load the selected lock into the Solve form for editing; saving replaces it.
    fn start_edit(&mut self) {
        let actual = match self.filtered().get(self.browse.selected).copied() {
            Some(i) => i,
            None => return,
        };
        let lock = self.locks[actual].clone();
        let n = lock.rules.len().clamp(MIN_TUMBLERS, MAX_TUMBLERS);
        let start = lock.start.clone().unwrap_or_else(|| vec![4; n]);
        let s = &mut self.solve;
        s.editing = Some(actual);
        s.n = n;
        s.rules = lock.rules.clone();
        s.rules.resize(n, String::new());
        s.name = lock.name.clone();
        s.start = start.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
        s.focus = 1;
        s.result = SolveResult::None;
        self.solve_anim = None;
        self.screen = Screen::Solve;
        self.status = format!("{} \"{}\"  ·  Enter, ^S", self.tr("browse.edit"), lock.name);
    }

    fn request_delete(&mut self) {
        let name = match self.selected_lock() {
            Some(l) => l.name.clone(),
            None => return,
        };
        self.browse.confirm_delete = true;
        self.status = format!("{} \"{}\"?  (y/n)", self.tr("browse.delete"), name);
    }

    fn delete_selected(&mut self) {
        self.browse.confirm_delete = false;
        let actual = match self.filtered().get(self.browse.selected).copied() {
            Some(i) => i,
            None => {
                self.status = self.tr("status.browse").to_string();
                return;
            }
        };
        match remove_lock_from_file(&self.file, actual) {
            Ok(name) => {
                self.locks = load_locks(&self.file);
                self.clamp_browse_selection();
                self.status = format!("{} \"{}\"", self.tr("browse.deleted"), name);
            }
            Err(e) => self.status = e,
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let n = self.filtered().len();
        if n == 0 {
            return;
        }
        let cur = self.browse.selected as i32;
        self.browse.selected = (cur + delta).rem_euclid(n as i32) as usize;
    }

    fn clamp_browse_selection(&mut self) {
        let n = self.filtered().len();
        if n == 0 {
            self.browse.selected = 0;
        } else if self.browse.selected >= n {
            self.browse.selected = n - 1;
        }
    }

    // Lock indices whose name matches the current filter (case-insensitive).
    fn filtered(&self) -> Vec<usize> {
        let q = self.browse.filter.to_lowercase();
        self.locks
            .iter()
            .enumerate()
            .filter(|(_, l)| q.is_empty() || l.name.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect()
    }

    fn selected_lock(&self) -> Option<&Lock> {
        let f = self.filtered();
        f.get(self.browse.selected).map(|&i| &self.locks[i])
    }

    fn enter_step_from_browse(&mut self) {
        let lock = match self.selected_lock() {
            Some(l) => l.clone(),
            None => return,
        };
        match step_from_lock(&lock) {
            Ok(s) => {
                self.step = Some(s);
                self.step_list = ListState::default();
                self.step_origin = Screen::Browse;
                self.screen = Screen::Step;
                self.status = self.tr("status.step").to_string();
            }
            Err(e) => self.status = format!("{}: {}", self.tr("msg.cantwalk"), e),
        }
    }

    fn on_key_solve(&mut self, key: KeyEvent) {
        if self.solve_anim.is_some() {
            self.solve_anim = None; // any key skips the cracking animation
            return;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let fields = self.solve.field_count();
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Browse;
                self.status = self.tr("status.browse").to_string();
            }
            KeyCode::Char('w') if ctrl => self.walk_solved(),
            KeyCode::Char('s') if ctrl => self.save_solved(),
            KeyCode::Tab | KeyCode::Down => self.solve.focus = (self.solve.focus + 1) % fields,
            KeyCode::BackTab | KeyCode::Up => {
                self.solve.focus = (self.solve.focus + fields - 1) % fields
            }
            // on the tumbler-count field, ←/→ adjust the count
            KeyCode::Left if self.solve.focus == 0 => self.solve.set_count(self.solve.n - 1),
            KeyCode::Right if self.solve.focus == 0 => self.solve.set_count(self.solve.n + 1),
            KeyCode::Enter => self.run_solve(),
            KeyCode::Backspace => {
                if let Some(field) = self.solve_field_mut() {
                    field.pop();
                }
            }
            // digit on the count field sets the count directly
            KeyCode::Char(c) if self.solve.focus == 0 && c.is_ascii_digit() => {
                self.solve.set_count(c.to_digit(10).unwrap() as usize)
            }
            KeyCode::Char(c) if !ctrl => {
                if let Some(field) = self.solve_field_mut() {
                    field.push(c);
                }
            }
            _ => {}
        }
    }

    // The editable text field at the current focus, or None for the count field.
    fn solve_field_mut(&mut self) -> Option<&mut String> {
        let n = self.solve.n;
        match self.solve.focus {
            0 => None, // tumbler count (not a text field)
            1 => Some(&mut self.solve.name),
            f if f >= 2 && f <= n + 1 => Some(&mut self.solve.rules[f - 2]),
            _ => Some(&mut self.solve.start),
        }
    }

    fn run_solve(&mut self) {
        let start = parse_start(&self.solve.start);
        if start.len() != self.solve.n {
            self.solve.result =
                SolveResult::Error(format!("{} ({})", self.tr("solve.err_count"), self.solve.n));
            return;
        }
        let mat = match build_matrix(&self.solve.rules) {
            Ok(m) => m,
            Err(e) => {
                self.solve.result = SolveResult::Error(e);
                return;
            }
        };
        match solve(&start, &mat) {
            None => {
                self.solve.result =
                    SolveResult::Error(self.tr("solve.err_nosolution").to_string());
            }
            Some(sol) => {
                let lines = nameless_locksmith::solution_lines(&sol);
                self.status = format!(
                    "{} {} {}",
                    self.tr("solve.solved_prefix"),
                    sol.total,
                    self.tr("solve.solved_suffix"),
                );
                self.solve.result = SolveResult::Solved {
                    total: sol.total,
                    lines,
                    mat,
                    start,
                    groups: sol.steps.clone(),
                };
                self.solve_anim = Some(0); // play the cracking animation
            }
        }
    }

    fn walk_solved(&mut self) {
        if let SolveResult::Solved {
            mat, start, groups, ..
        } = &self.solve.result
        {
            let name = if self.solve.name.trim().is_empty() {
                self.tr("value.unnamed").to_string()
            } else {
                self.solve.name.trim().to_string()
            };
            self.step = Some(StepState::new(name, mat.clone(), start.clone(), groups.clone()));
            self.step_list = ListState::default();
            self.step_origin = Screen::Solve;
            self.screen = Screen::Step;
            self.status = self.tr("status.step").to_string();
        } else {
            self.status = self.tr("solve.nothing_walk").to_string();
        }
    }

    fn save_solved(&mut self) {
        let (total, start, groups) = match &self.solve.result {
            SolveResult::Solved {
                total, start, groups, ..
            } => (*total, start.clone(), groups.clone()),
            _ => {
                self.status = self.tr("solve.nothing_save").to_string();
                return;
            }
        };
        let name = if self.solve.name.trim().is_empty() {
            self.tr("value.unnamed").to_string()
        } else {
            self.solve.name.trim().to_string()
        };
        let sol = Solution {
            total,
            steps: groups,
        };
        let rules: Vec<String> = self.solve.rules[..self.solve.n].to_vec();
        let result = match self.solve.editing {
            // editing an existing lock → replace it in place
            Some(idx) => replace_lock_in_file(&self.file, idx, &name, &rules, &start, &sol)
                .map(|()| self.tr("msg.updated").to_string()),
            None => append_lock(&self.file, &name, &rules, &start, &sol)
                .map(|()| self.tr("msg.saved").to_string()),
        };
        match result {
            Ok(verb) => {
                self.status = format!("{} \"{}\" → {}", verb, name, self.file);
                self.locks = load_locks(&self.file); // refresh browse list
            }
            Err(e) => self.status = e,
        }
    }

    fn on_key_step(&mut self, key: KeyEvent) {
        let origin = self.step_origin;
        let step = match self.step.as_mut() {
            Some(s) => s,
            None => {
                self.screen = origin;
                return;
            }
        };
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('b') => {
                self.screen = origin;
                self.status = match origin {
                    Screen::Solve => self.tr("status.solve").to_string(),
                    _ => self.tr("status.browse").to_string(),
                };
            }
            KeyCode::Right | KeyCode::Char(' ') | KeyCode::Char('l') | KeyCode::Char('n') => {
                if step.idx < step.clicks.len() {
                    step.idx += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('p') => {
                if step.idx > 0 {
                    step.idx -= 1;
                }
            }
            KeyCode::Home | KeyCode::Char('g') => step.idx = 0,
            KeyCode::End | KeyCode::Char('G') => step.idx = step.clicks.len(),
            _ => {}
        }
    }

    // ---------- rendering ----------

    fn draw(&mut self, f: &mut Frame) {
        let [title, body, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(f.area());

        let tabs = Line::from(vec![
            " nameless-locksmith ".bold().bg(Color::Blue).fg(Color::White),
            "  ".into(),
            tab_span(self.tr("tab.browse"), self.screen == Screen::Browse),
            " ".into(),
            tab_span(self.tr("tab.solve"), self.screen == Screen::Solve),
            " ".into(),
            tab_span(self.tr("tab.step"), self.screen == Screen::Step),
        ]);
        f.render_widget(Paragraph::new(tabs), title);

        match self.screen {
            Screen::Language => self.draw_language(f, body),
            Screen::Browse => self.draw_browse(f, body),
            Screen::Solve => self.draw_solve(f, body),
            Screen::Step => self.draw_step(f, body),
        }

        f.render_widget(
            Paragraph::new(Line::from(self.status.clone()).fg(Color::DarkGray)),
            footer,
        );
    }

    fn draw_language(&mut self, f: &mut Frame, area: Rect) {
        let height = (LANGUAGES.len() + 5) as u16;
        let [box_area] = Layout::vertical([Constraint::Length(height)])
            .flex(Flex::Center)
            .areas(area);
        let [box_area] = Layout::horizontal([Constraint::Length(40)])
            .flex(Flex::Center)
            .areas(box_area);

        let mut lines =
            vec![Line::from("Select language / Wybierz język".bold()), Line::from("")];
        for (i, (_, name, _)) in LANGUAGES.iter().enumerate() {
            lines.push(if i == self.lang_sel {
                Line::from(format!("  ▶ {}  ", name))
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .bold()
            } else {
                Line::from(format!("    {}  ", name)).fg(Color::White)
            });
        }
        lines.push(Line::from(""));
        lines.push(Line::from("↑↓ + Enter   ·   e / p".fg(Color::DarkGray)));
        f.render_widget(
            Paragraph::new(lines).block(Block::bordered().title(" Język / Language ")),
            box_area,
        );
    }

    fn draw_browse(&mut self, f: &mut Frame, area: Rect) {
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)])
                .areas(area);
        let [filter_area, list_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(left);

        // filter box
        let filter_style = if self.browse.filtering {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new().fg(Color::DarkGray)
        };
        let filter_text = if self.browse.filter.is_empty() && !self.browse.filtering {
            self.tr("filter.placeholder").to_string()
        } else {
            format!(
                "{}{}",
                self.browse.filter,
                if self.browse.filtering { "▏" } else { "" }
            )
        };
        let filter_title = self.tr("filter.title").to_string();
        f.render_widget(
            Paragraph::new(filter_text)
                .style(filter_style)
                .block(Block::bordered().title(filter_title)),
            filter_area,
        );

        // list
        let indices = self.filtered();
        let items: Vec<ListItem> = indices
            .iter()
            .map(|&i| {
                let l = &self.locks[i];
                let mark = if l.solution.is_empty() { " " } else { "✓" };
                ListItem::new(format!("[{}] {}", mark, l.name))
            })
            .collect();
        self.browse.list_state.select(if indices.is_empty() {
            None
        } else {
            Some(self.browse.selected.min(indices.len() - 1))
        });
        let list_title = format!("{} ({})", self.tr("locks.title"), indices.len());
        let list = List::new(items)
            .block(Block::bordered().title(list_title))
            .highlight_style(
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        f.render_stateful_widget(list, list_area, &mut self.browse.list_state);

        // detail
        let detail = match self.selected_lock() {
            Some(l) => lock_detail_lines(l, &self.i18n),
            None => vec![Line::from(self.tr("detail.empty").to_string().fg(Color::DarkGray))],
        };
        let detail_title = self.tr("detail.title").to_string();
        f.render_widget(
            Paragraph::new(detail)
                .wrap(Wrap { trim: false })
                .block(Block::bordered().title(detail_title)),
            right,
        );
    }

    fn draw_solve(&mut self, f: &mut Frame, area: Rect) {
        let n = self.solve.n;
        // left column: the form on top, the format/example help below it;
        // right column: the result (and the cracking animation).
        let [left, result] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(area);
        let [form, help] =
            Layout::vertical([Constraint::Length((n + 5) as u16), Constraint::Fill(1)]).areas(left);

        let mut lines: Vec<Line> = Vec::new();
        lines.push(field_line(
            self.tr("solve.tumblers"),
            &format!("{}   ◄ ►", n),
            self.solve.focus == 0,
        ));
        lines.push(field_line(self.tr("field.name"), &self.solve.name, self.solve.focus == 1));
        for i in 0..n {
            lines.push(field_line(
                &format!("{} {}", self.tr("field.rule"), i + 1),
                &self.solve.rules[i],
                self.solve.focus == i + 2,
            ));
        }
        lines.push(field_line("Start", &self.solve.start, self.solve.focus == n + 2));
        lines.push(Line::from(self.tr("solve.hint").to_string().fg(Color::DarkGray)));
        let solve_title = self.tr("solve.title").to_string();
        f.render_widget(
            Paragraph::new(lines).block(Block::bordered().title(solve_title)),
            form,
        );

        // ----- always-visible format & examples panel -----
        let dim = Color::DarkGray;
        let help_lines: Vec<Line> = vec![
            Line::from(format!(" {}", self.tr("help.example"))).fg(Color::Cyan),
            Line::from("   1: 3r, 6l   2: -   3: 1r, 4l, 6r").fg(Color::White),
            Line::from("   4: 2r, 5r, 6l   5: -   6: 3l").fg(Color::White),
            Line::from(format!("   Start: 5,3,6,7,2,7  → {}", self.tr("help.solves")))
                .fg(Color::White),
            Line::from(""),
            Line::from(strip_tags(self.tr("help.dirs"))).fg(dim),
            Line::from(strip_tags(self.tr("help.start"))).fg(dim),
            Line::from(strip_tags(self.tr("help.output"))).fg(dim),
            Line::from(strip_tags(self.tr("help.tip"))).fg(dim),
        ];
        f.render_widget(
            Paragraph::new(help_lines)
                .wrap(Wrap { trim: false })
                .block(Block::bordered().title(self.tr("solve.format").to_string())),
            help,
        );

        // While the cracking animation plays, the result area shows the lock
        // spinning its plates and settling onto hole 4 — then the steps appear.
        if let Some(frame) = self.solve_anim {
            let pos = anim_positions(frame, n);
            let cracked = pos.iter().all(|&p| p == 4);
            let mut lines: Vec<Line> = Vec::new();
            for t in (0..n).rev() {
                lines.push(plate_line(t, pos[t], !cracked && pos[t] != 4));
            }
            lines.push(if cracked {
                Line::from(self.tr("step.open").to_string().fg(Color::Green).bold())
            } else {
                Line::from(format!("{} …", self.tr("solve.cracking")).fg(Color::Yellow).bold())
            });
            f.render_widget(
                Paragraph::new(lines).block(Block::bordered().title(self.tr("result.title").to_string())),
                result,
            );
            return;
        }

        let result_lines: Vec<Line> = match &self.solve.result {
            SolveResult::None => {
                vec![Line::from(self.tr("result.empty").to_string().fg(Color::DarkGray))]
            }
            SolveResult::Error(e) => vec![Line::from(format!("✗ {}", e).fg(Color::Red))],
            SolveResult::Solved { total, lines, .. } => {
                let mut out = vec![Line::from(
                    format!("✓ {} {}", total, self.tr("result.solved_suffix"))
                        .fg(Color::Green)
                        .bold(),
                )];
                for line in lines {
                    out.push(Line::from(format!("  {}", line)));
                }
                out
            }
        };
        let result_title = self.tr("result.title").to_string();
        f.render_widget(
            Paragraph::new(result_lines)
                .wrap(Wrap { trim: false })
                .block(Block::bordered().title(result_title)),
            result,
        );
    }

    fn draw_step(&mut self, f: &mut Frame, area: Rect) {
        // Snapshot everything we need so we don't hold a borrow of self.step
        // while mutating self.step_list below.
        let (name, pos, groups, idx, nclicks, cur) = match &self.step {
            Some(s) => (
                s.name.clone(),
                s.positions(),
                s.groups.clone(),
                s.idx,
                s.clicks.len(),
                s.current_group(),
            ),
            None => return,
        };
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)])
                .areas(area);

        // ----- aligned plate stack (left) -----
        let n = pos.len();
        let target = cur.map(|g| groups[g].0); // tumbler of the next click ("selected" plate)
        let solved = pos.iter().all(|&p| p == 4) && idx == nclicks;
        let mut track_lines: Vec<Line> = Vec::new();
        // fixed centre guide: every pin sits in PIN_COL, the goal is hole 4 here
        let mut pointer = vec![Span::raw(" ".repeat(PIN_COL)), "▼".fg(Color::Cyan).bold()];
        pointer.push("  4".fg(Color::Cyan));
        track_lines.push(Line::from(pointer));
        track_lines.push(Line::from(format!(" {}", self.tr("step.goal")).fg(Color::DarkGray)));
        for t in (0..n).rev() {
            track_lines.push(plate_line(t, pos[t], !solved && target == Some(t + 1)));
        }
        track_lines.push(Line::from(""));
        let centered = pos.iter().filter(|&&p| p == 4).count();
        track_lines.push(if solved {
            Line::from(self.tr("step.open").to_string().fg(Color::Green).bold())
        } else {
            Line::from(vec![
                format!("{} {} / {}   ", self.tr("step.click"), idx, nclicks).fg(Color::White),
                format!("· {}/{} {}", centered, n, self.tr("step.pins")).fg(Color::DarkGray),
            ])
        });
        let lock_title = format!("{} — {}", self.tr("step.lock"), name);
        f.render_widget(
            Paragraph::new(track_lines).block(Block::bordered().title(lock_title)),
            left,
        );

        // ----- right column: big current-move panel over a scrolling step list -----
        let [move_area, steps_area] =
            Layout::vertical([Constraint::Length(7), Constraint::Fill(1)]).areas(right);

        let key_col = Color::Rgb(255, 190, 70);
        let move_lines: Vec<Line> = if solved {
            vec![
                Line::from(""),
                Line::from("✓ OPEN".fg(Color::Green).bold()),
                Line::from(""),
                Line::from("◉ ◉ ◉ ◉ ◉ ◉".fg(Color::Green).bold()),
            ]
        } else if let Some(g) = cur {
            let (t, k, n) = groups[g];
            let cap = keycap(k);
            // direction arrows: one per press, D = plate right, A = plate left
            let arrow = if k == 'D' { "→" } else { "←" };
            let arrows = vec![arrow; n].join(" ");
            // Keep the three keycap lines the same width so centering aligns them;
            // the count + arrows go on their own line below.
            vec![
                Line::from(format!("▶ {} {}", self.tr("step.tumbler"), t).fg(Color::Yellow).bold()),
                Line::from(cap[0].clone().fg(key_col)),
                Line::from(cap[1].clone().fg(key_col).bold()),
                Line::from(cap[2].clone().fg(key_col)),
                Line::from(format!("× {}   {}", n, arrows).fg(Color::White).bold()),
            ]
        } else {
            vec![]
        };
        let move_title = format!("{} ({}/{})", self.tr("step.current"), sel_index(cur, &groups) + 1, groups.len());
        f.render_widget(
            Paragraph::new(move_lines)
                .centered()
                .block(Block::bordered().title(move_title)),
            move_area,
        );

        // ----- step list (scrolls to follow the current step) -----
        let items: Vec<ListItem> = groups
            .iter()
            .enumerate()
            .map(|(g, &(t, k, n))| {
                let (mark, style) = if Some(g) == cur {
                    (
                        "▶ ",
                        Style::new()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                } else if cur.map(|c| g < c).unwrap_or(true) {
                    ("✓ ", Style::new().fg(Color::DarkGray)) // already done
                } else {
                    ("  ", Style::new().fg(Color::White))
                };
                ListItem::new(Line::from(format!("{}{}   {}× {}", mark, t, n, k)).style(style))
            })
            .collect();
        // Follow the current step (or the last one when complete) so long
        // solutions scroll into view.
        let sel = sel_index(cur, &groups);
        self.step_list.select(Some(sel));
        let steps_title = format!("{} ({}/{})", self.tr("step.steps"), sel + 1, groups.len());
        let list = List::new(items).block(Block::bordered().title(steps_title));
        f.render_stateful_widget(list, steps_area, &mut self.step_list);
    }
}

// ---------- helpers ----------

// The step index to highlight/scroll to: the current group, or the last one
// when the walk is complete.
fn sel_index(cur: Option<usize>, groups: &[(usize, char, usize)]) -> usize {
    cur.unwrap_or(groups.len().saturating_sub(1))
}

fn load_locks(file: &str) -> Vec<Lock> {
    match std::fs::read_to_string(file) {
        Ok(text) => parse_history(&text),
        Err(_) => Vec::new(),
    }
}

fn step_from_lock(lock: &Lock) -> Result<StepState, String> {
    let start = lock.start.clone().ok_or("no start position recorded")?;
    let groups = parse_solution_steps(&lock.solution, lock.rules.len())
        .ok_or("no step-by-step solution recorded")?;
    let mat = build_matrix(&lock.rules)?;
    Ok(StepState::new(lock.name.clone(), mat, start, groups))
}

// All integers in a free-form string (commas/spaces/brackets ok).
fn parse_start(s: &str) -> Vec<i32> {
    s.split(|c: char| !c.is_ascii_digit() && c != '-')
        .filter(|x| !x.is_empty())
        .filter_map(|x| x.parse().ok())
        .collect()
}

// Remove HTML tags from a localized help string (the help.* strings are written
// with <b>/<i>/<code> markup for the web; the TUI shows them as plain text).
fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

fn tab_span(label: &str, active: bool) -> Span<'static> {
    let s = format!(" {} ", label);
    if active {
        s.fg(Color::Black).bg(Color::Cyan).bold()
    } else {
        s.fg(Color::DarkGray)
    }
}

fn field_line<'a>(label: &str, value: &str, focused: bool) -> Line<'a> {
    let lbl = format!(" {:<7} ", label);
    let val = if focused {
        format!("{}▏", value)
    } else {
        value.to_string()
    };
    let label_span = if focused {
        lbl.fg(Color::Black).bg(Color::Cyan).bold()
    } else {
        lbl.fg(Color::Cyan)
    };
    let val_span = if focused {
        val.fg(Color::Yellow)
    } else {
        val.fg(Color::White)
    };
    Line::from(vec![label_span, "  ".into(), val_span])
}

fn lock_detail_lines(l: &Lock, i18n: &I18n) -> Vec<Line<'static>> {
    let mut out = vec![Line::from(l.name.clone().bold().fg(Color::Cyan)), Line::from("")];
    out.push(Line::from(
        format!("{} · {} {}", i18n.get("label.rules"), l.rules.len(), i18n.get("step.tumbler"))
            .fg(Color::DarkGray),
    ));
    for (i, r) in l.rules.iter().enumerate() {
        let r = if r.is_empty() { "-" } else { r };
        out.push(Line::from(format!("  {}: {}", i + 1, r)));
    }
    out.push(Line::from(""));
    match &l.start {
        Some(s) => out.push(Line::from(format!(
            "Start  [{}]",
            s.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
        ))),
        None => out.push(Line::from(
            format!("Start  {}", i18n.get("value.none")).fg(Color::DarkGray),
        )),
    }
    out.push(Line::from(""));
    if l.solution.is_empty() {
        out.push(Line::from(
            format!("{} {}", i18n.get("label.solution"), i18n.get("value.none"))
                .fg(Color::DarkGray),
        ));
    } else {
        out.push(Line::from(
            format!(
                "{} ({} {})",
                i18n.get("label.solution"),
                l.solution.len(),
                i18n.get("label.steps")
            )
            .fg(Color::DarkGray),
        ));
        for step in &l.solution {
            out.push(Line::from(format!("  {}", step)));
        }
    }
    out
}

// Column (0-based) where every plate's PIN sits, so the pins line up vertically
// across all tumblers. = marker(2) + label(2) + CENTER slots * 2 chars each.
const PIN_COL: usize = 2 + 2 + (PLATE_CENTER as usize) * 2;
// 15 slots with the pin at slot 7 keeps BOTH plate edges on screen for every
// position 1..7 (the plate spans 9 cells and slides ±6 around the centre).
const PLATE_SLOTS: i32 = 15;
const PLATE_CENTER: i32 = 7; // the fixed pin column (slot index)

// One tumbler plate. The PIN is fixed at a shared centre column (PIN_COL); the
// row of holes (and the goal hole, position 4) slides past it as `p` changes —
// matching the in-game animation where the plate moves and the pin stays put.
// Plates are NOT staggered, so two tumblers at the same position render
// identically and line up; a solved lock is one clean vertical column of pins on
// hole 4. Depth is shown by shading alone: tumbler 1 (t=0) is brightest/front.
fn plate_line(t: usize, p: i32, is_target: bool) -> Line<'static> {
    let depth = t as u8; // 0 = front/bright, 5 = back/dim
    let shade = 235u8.saturating_sub(depth * 20);
    let plate = Color::Rgb(shade, shade, shade);
    let hole = Color::Rgb(shade / 3, shade / 3, shade / 3);
    let brass = Color::Rgb(
        225u8.saturating_sub(depth * 16),
        155u8.saturating_sub(depth * 12),
        65u8.saturating_sub(depth * 8),
    );

    // marker column: the active plate gets a ▶ so it's easy to spot
    let mut spans: Vec<Span> = vec![if is_target {
        "▶ ".fg(Color::Yellow).bold()
    } else {
        "  ".into()
    }];
    let label = format!("{} ", t + 1);
    spans.push(if is_target {
        label.fg(Color::Yellow).bold()
    } else {
        label.fg(plate)
    });

    for c in 0..PLATE_SLOTS {
        let h = p + (c - PLATE_CENTER); // hole of the plate under slot `c`
        let glyph = if c == PLATE_CENTER {
            // the fixed pin, sitting over hole `p`
            if p == 4 {
                "◉".fg(Color::Green).bold()
            } else if is_target {
                "◉".fg(Color::Rgb(255, 190, 70)).bold()
            } else {
                "◉".fg(brass).bold()
            }
        } else if h == 4 {
            "◌".fg(Color::Rgb(90, 180, 180)) // the goal hole, sliding toward the pin
        } else if (1..=7).contains(&h) {
            "○".fg(hole)
        } else if h == 0 {
            "▕".fg(plate) // left edge of the plate
        } else if h == 8 {
            "▏".fg(plate) // right edge of the plate
        } else {
            " ".into() // off the plate (lock interior)
        };
        spans.push(glyph);
        spans.push(" ".into());
    }

    if p == 4 {
        spans.push("✓".fg(Color::Green).bold());
    } else {
        spans.push(format!("{}", p).fg(plate));
    }
    Line::from(spans)
}

// A three-row keycap around a key letter, e.g.  ╭───╮ / │ D │ / ╰───╯
fn keycap(k: char) -> [String; 3] {
    [
        "╭───╮".to_string(),
        format!("│ {} │", k),
        "╰───╯".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use nameless_locksmith::goal;
    use ratatui::{backend::TestBackend, Terminal};

    // Every key present in English must also exist in every other language, and
    // vice-versa — guards against half-translated or stale catalogs.
    #[test]
    fn catalogs_have_matching_keys() {
        let en = parse_catalog(LANGUAGES[0].2);
        assert!(!en.is_empty());
        for &(code, _, src) in &LANGUAGES[1..] {
            let cat = parse_catalog(src);
            for k in en.keys() {
                assert!(cat.contains_key(k), "{}: missing key '{}'", code, k);
            }
            for k in cat.keys() {
                assert!(en.contains_key(k), "{}: unknown key '{}' (not in en)", code, k);
            }
        }
    }

    // Render the Step screen at a few points in the walk and dump it. Guards the
    // draw path and lets us eyeball the plate stack with
    // `cargo test render_step -- --nocapture`.
    #[test]
    fn render_step_screen() {
        let mut app = App::new("history-of-locks.md");
        let lock = app.locks[0].clone(); // lock 1
        app.step = Some(step_from_lock(&lock).expect("lock 1 is walkable"));
        app.screen = Screen::Step;

        let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
        for &at in &[0usize, 8, 35] {
            if let Some(s) = app.step.as_mut() {
                s.idx = at.min(s.clicks.len());
            }
            terminal.draw(|f| app.draw(f)).unwrap();
            println!("\n--- step idx = {} ---", at);
            println!("{}", terminal.backend());
        }
    }

    fn key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    // Both plate edges must stay on screen for every pin position 1..7.
    #[test]
    fn plate_edges_visible_at_every_position() {
        for p in 1..=7 {
            let line = plate_line(0, p, false);
            let s: String = line.spans.iter().map(|sp| sp.content.as_ref()).collect();
            assert!(s.contains('▕'), "left edge missing at position {}", p);
            assert!(s.contains('▏'), "right edge missing at position {}", p);
        }
    }

    // Pressing 'd' then 'y' in Browse removes the selected lock from the file.
    #[test]
    fn delete_flow_removes_lock_from_file() {
        use std::fs;
        let src = fs::read_to_string("history-of-locks.md").unwrap();
        let path = std::env::temp_dir().join("nlsmith-delete-flow-test.md");
        let path = path.to_str().unwrap().to_string();
        fs::write(&path, &src).unwrap();

        let mut app = App::new(&path);
        let before = app.locks.len();
        assert!(before >= 2);
        app.confirm_language(); // -> Browse
        app.browse.selected = before - 1; // last lock

        app.on_key(key('d')); // request delete
        assert!(app.browse.confirm_delete);
        assert!(app.status.contains("Delete"));

        app.on_key(key('n')); // cancel — nothing removed
        assert!(!app.browse.confirm_delete);
        assert_eq!(app.locks.len(), before);

        app.on_key(key('d'));
        app.on_key(key('y')); // confirm
        assert!(!app.browse.confirm_delete);
        assert_eq!(app.locks.len(), before - 1);
        assert_eq!(parse_history(&fs::read_to_string(&path).unwrap()).len(), before - 1);

        fs::remove_file(&path).ok();
    }

    // Render Browse and Solve for eyeballing (and as draw-path guards):
    //   cargo test render_screens -- --nocapture
    #[test]
    fn render_screens() {
        let mut app = App::new("history-of-locks.md");
        app.confirm_language();
        let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();

        app.screen = Screen::Browse;
        app.browse.selected = 4;
        terminal.draw(|f| app.draw(f)).unwrap();
        println!("\n--- Browse ---\n{}", terminal.backend());

        app.screen = Screen::Solve;
        app.solve.name = "Vault behind the inn".into();
        app.solve.rules = ["3r, 6l", "-", "1r, 4l, 6r", "2r, 5r, 6l", "-", "3l"]
            .map(String::from)
            .to_vec();
        app.solve.start = "5, 3, 6, 7, 2, 7".into();
        app.run_solve();
        terminal.draw(|f| app.draw(f)).unwrap();
        println!("\n--- Solve ---\n{}", terminal.backend());
    }

    // The cracking animation spins then settles all plates onto hole 4.
    //   cargo test render_solve_animation -- --nocapture
    #[test]
    fn render_solve_animation() {
        assert_ne!(anim_positions(0, 6), goal(6), "frame 0 should be mid-spin");
        assert_eq!(anim_positions(ANIM_END, 6), goal(6), "final frame should be solved");

        let mut app = App::new("history-of-locks.md");
        app.confirm_language();
        app.screen = Screen::Solve;
        app.solve.rules = ["3r, 6l", "-", "1r, 4l, 6r", "2r, 5r, 6l", "-", "3l"]
            .map(String::from)
            .to_vec();
        app.solve.start = "5, 3, 6, 7, 2, 7".into();
        app.run_solve();
        assert_eq!(app.solve_anim, Some(0)); // solving kicks off the animation

        let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
        for &f in &[2u32, 13, ANIM_END] {
            app.solve_anim = Some(f);
            terminal.draw(|t| app.draw(t)).unwrap();
            println!("\n--- anim frame {} ---\n{}", f, terminal.backend());
        }
    }

    // Step-through renders for a non-6 tumbler lock without panicking.
    #[test]
    fn render_step_three_tumblers() {
        let mat = build_matrix(&["2r".to_string(), "-".to_string(), "1l".to_string()]).unwrap();
        let sol = solve(&[6, 2, 5], &mat).unwrap();
        let mut app = App::new("history-of-locks.md");
        app.confirm_language();
        app.step = Some(StepState::new("mini".into(), mat, vec![6, 2, 5], sol.steps));
        app.screen = Screen::Step;
        let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
        if let Some(s) = app.step.as_mut() {
            s.idx = s.clicks.len(); // walk to the end (solved)
        }
        terminal.draw(|f| app.draw(f)).unwrap();
        let dump = format!("{}", terminal.backend());
        assert!(dump.contains("OPEN"), "3-tumbler lock should show solved state");
    }

    // Pressing 'e' in Browse loads a lock into Solve; re-solving + ^S replaces
    // it in the history file (no duplicate, same count).
    #[test]
    fn edit_flow_replaces_lock_in_file() {
        use std::fs;
        let src = fs::read_to_string("history-of-locks.md").unwrap();
        let path = std::env::temp_dir().join("nlsmith-edit-flow-test.md");
        let path = path.to_str().unwrap().to_string();
        fs::write(&path, &src).unwrap();

        let mut app = App::new(&path);
        let before = app.locks.len();
        app.confirm_language();
        app.browse.selected = 0; // first lock

        app.on_key(key('e')); // load into Solve for editing
        assert_eq!(app.screen, Screen::Solve);
        assert_eq!(app.solve.editing, Some(0));
        assert!(!app.solve.rules[0].is_empty()); // form populated

        app.run_solve(); // re-solve the loaded lock
        app.save_solved(); // replace in place
        assert!(app.status.contains("Updated"));
        // count unchanged; file still parses to the same number of locks
        assert_eq!(app.locks.len(), before);
        assert_eq!(parse_history(&fs::read_to_string(&path).unwrap()).len(), before);

        fs::remove_file(&path).ok();
    }

    // The language picker renders, and selecting Polski switches the catalog.
    #[test]
    fn language_pick_switches_catalog() {
        let mut app = App::new("history-of-locks.md");
        let mut terminal = Terminal::new(TestBackend::new(80, 22)).unwrap();
        terminal.draw(|f| app.draw(f)).unwrap();
        println!("{}", terminal.backend());

        app.lang_sel = 1;
        app.confirm_language();
        assert_eq!(LANGUAGES[app.lang_sel].0, "pl");
        assert_eq!(app.screen, Screen::Browse);
        assert_eq!(app.tr("tab.browse"), "Przeglądaj");
    }
}
