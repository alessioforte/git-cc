use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tui_textarea::TextArea;

use crate::data::emojis::EMOJIS;
use crate::data::kinds::KINDS;
use crate::settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Kind,
    Scope,
    Emoji,
    Subject,
    Description,
    Breaking,
    Confirm,
}

impl Step {
    pub const ALL: [Step; 7] = [
        Step::Kind,
        Step::Scope,
        Step::Emoji,
        Step::Subject,
        Step::Description,
        Step::Breaking,
        Step::Confirm,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Step::Kind => "Kind",
            Step::Scope => "Scope",
            Step::Emoji => "Emoji",
            Step::Subject => "Subject",
            Step::Description => "Description",
            Step::Breaking => "Breaking",
            Step::Confirm => "Confirm",
        }
    }

    pub fn index(&self) -> usize {
        Self::ALL.iter().position(|s| s == self).unwrap()
    }

    pub fn next(&self) -> Option<Step> {
        let i = self.index();
        Self::ALL.get(i + 1).copied()
    }

    pub fn prev(&self) -> Option<Step> {
        let i = self.index();
        if i > 0 {
            Some(Self::ALL[i - 1])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
}

pub struct SelectState {
    pub items: Vec<(String, String)>, // (value, display_label)
    pub filtered: Vec<usize>,
    pub cursor: usize,
    pub filter: String,
    pub offset: usize,
    pub mode: InputMode,
}

impl SelectState {
    pub fn new(items: Vec<(String, String)>) -> Self {
        let filtered: Vec<usize> = (0..items.len()).collect();
        Self {
            items,
            filtered,
            cursor: 0,
            filter: String::new(),
            offset: 0,
            mode: InputMode::Normal,
        }
    }

    pub fn apply_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered = (0..self.items.len()).collect();
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored: Vec<(usize, i64)> = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, (val, label))| {
                    let haystack = format!("{} {}", val, label);
                    matcher
                        .fuzzy_match(&haystack, &self.filter)
                        .map(|score| (i, score))
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered = scored.into_iter().map(|(i, _)| i).collect();
        }
        if self.filtered.is_empty() {
            self.cursor = 0;
        } else if self.cursor >= self.filtered.len() {
            self.cursor = self.filtered.len() - 1;
        }
        self.offset = 0;
    }

    pub fn move_up(&mut self) {
        if !self.filtered.is_empty() {
            if self.cursor > 0 {
                self.cursor -= 1;
            } else {
                self.cursor = self.filtered.len() - 1;
            }
            self.adjust_offset();
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() {
            if self.cursor < self.filtered.len() - 1 {
                self.cursor += 1;
            } else {
                self.cursor = 0;
            }
            self.adjust_offset();
        }
    }

    fn adjust_offset(&mut self) {
        // Keep cursor visible within a reasonable window
        if self.cursor < self.offset {
            self.offset = self.cursor;
        }
    }

    pub fn adjust_offset_for_height(&mut self, height: usize) {
        if height == 0 {
            return;
        }
        if self.cursor >= self.offset + height {
            self.offset = self.cursor - height + 1;
        }
        if self.cursor < self.offset {
            self.offset = self.cursor;
        }
    }

    pub fn selected_value(&self) -> Option<&str> {
        if self.filtered.is_empty() {
            None
        } else {
            Some(&self.items[self.filtered[self.cursor]].0)
        }
    }

    pub fn push_char(&mut self, c: char) {
        self.filter.push(c);
        self.apply_filter();
    }

    pub fn pop_char(&mut self) {
        self.filter.pop();
        self.apply_filter();
    }

    pub fn reset_filter(&mut self) {
        self.filter.clear();
        self.apply_filter();
    }

    pub fn enter_insert(&mut self) {
        self.mode = InputMode::Insert;
    }

    pub fn exit_insert(&mut self) {
        self.mode = InputMode::Normal;
        self.reset_filter();
    }

    pub fn select_by_value(&mut self, value: &str) {
        if let Some(pos) = self
            .filtered
            .iter()
            .position(|&i| self.items[i].0 == value)
        {
            self.cursor = pos;
        }
    }
}

pub struct App<'a> {
    pub step: Step,
    pub running: bool,
    pub committed: bool,

    // Field values
    pub kind: Option<String>,
    pub scope: Option<String>,
    pub emoji: Option<String>,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub breaking: Option<String>,

    // Per-step UI state
    pub kind_select: SelectState,
    pub scope_select: SelectState,
    pub scope_input_mode: bool,
    pub scope_input: String,
    pub scope_save: bool,
    pub emoji_select: SelectState,
    pub subject_input: String,
    pub subject_error: bool,
    pub subject_mode: InputMode,
    pub desc_textarea: TextArea<'a>,
    pub desc_mode: InputMode,
    pub breaking_textarea: TextArea<'a>,
    pub breaking_mode: InputMode,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        // Build kind items
        let kind_items: Vec<(String, String)> = KINDS
            .iter()
            .map(|k| {
                (
                    k.value.to_string(),
                    format!("{:<10} {}", k.label, k.description),
                )
            })
            .collect();

        // Build scope items from settings
        let settings = settings::get_settings();
        let mut scope_items: Vec<(String, String)> = vec![("".to_string(), "None".to_string())];
        for s in &settings.scopes {
            scope_items.push((s.clone(), s.clone()));
        }
        scope_items.push((
            "__new_scope__".to_string(),
            "New scope".to_string(),
        ));
        scope_items.push((
            "__new_scope_once__".to_string(),
            "New scope (only use once)".to_string(),
        ));

        // Build emoji items
        let emoji_items: Vec<(String, String)> = EMOJIS
            .iter()
            .map(|e| {
                (
                    e.code.to_string(),
                    format!("{}  {}", e.emoji, e.description),
                )
            })
            .collect();

        let mut desc_textarea = TextArea::default();
        desc_textarea.set_placeholder_text("Enter a longer description (optional)");

        let mut breaking_textarea = TextArea::default();
        breaking_textarea.set_placeholder_text("BREAKING CHANGE: describe what breaks (optional)");

        Self {
            step: Step::Kind,
            running: true,
            committed: false,

            kind: None,
            scope: None,
            emoji: None,
            subject: None,
            description: None,
            breaking: None,

            kind_select: SelectState::new(kind_items),
            scope_select: SelectState::new(scope_items),
            scope_input_mode: false,
            scope_input: String::new(),
            scope_save: false,
            emoji_select: SelectState::new(emoji_items),
            subject_input: String::new(),
            subject_error: false,
            subject_mode: InputMode::Normal,
            desc_textarea,
            desc_mode: InputMode::Normal,
            breaking_textarea,
            breaking_mode: InputMode::Normal,
        }
    }

    pub fn go_next(&mut self) {
        if let Some(next) = self.step.next() {
            self.step = next;
        }
    }

    pub fn go_back(&mut self) {
        if let Some(prev) = self.step.prev() {
            self.step = prev;
            match prev {
                Step::Kind => {
                    self.kind_select.reset_filter();
                    if let Some(ref v) = self.kind {
                        self.kind_select.select_by_value(v);
                    }
                }
                Step::Scope => {
                    self.scope_input_mode = false;
                    self.scope_select.reset_filter();
                }
                Step::Emoji => {
                    self.emoji_select.reset_filter();
                    if let Some(ref v) = self.emoji {
                        self.emoji_select.select_by_value(v);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn formatted_commit(&self) -> String {
        crate::formatters::format_commit(
            self.kind.as_deref().unwrap_or(""),
            self.scope.as_deref().unwrap_or(""),
            self.emoji.as_deref().unwrap_or(""),
            self.subject.as_deref().unwrap_or(""),
            self.description.clone(),
            self.breaking.clone(),
        )
    }
}
