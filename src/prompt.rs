use colored::{Color, ColoredString, Colorize};
use reedline::{PromptEditMode, PromptHistorySearchStatus, PromptViMode};
use std::borrow::Cow;

static PROMPT_INDICATOR: &str = ": ";
static MULTILINE_INDICATOR: &str = "::: ";
static VI_INSERT_PROMPT_INDICATOR: &str = "[i]: ";
static VI_NORMAL_PROMPT_INDICATOR: &str = "[n]: ";

pub struct Prompt(ColoredString);

impl Prompt {
    pub fn new(username: &str, foreground: Color) -> Self {
        Self(username.color(foreground))
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self::new(whoami::username().as_str(), Color::Cyan)
    }
}

impl reedline::Prompt for Prompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Borrowed(&self.0)
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, edit_mode: PromptEditMode) -> Cow<str> {
        match edit_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => PROMPT_INDICATOR.into(),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Normal => VI_NORMAL_PROMPT_INDICATOR.into(),
                PromptViMode::Insert => VI_INSERT_PROMPT_INDICATOR.into(),
            },
            PromptEditMode::Custom(str) => format!("({str})").into(),
        }
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed(MULTILINE_INDICATOR)
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };

        // NOTE: magic strings, given there is logic on how these compose I am not sure if it
        // is worth extracting in to static constant
        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }

    fn get_prompt_color(&self) -> reedline::Color {
        reedline::Color::Cyan
    }

    fn get_indicator_color(&self) -> reedline::Color {
        reedline::Color::White
    }
}
