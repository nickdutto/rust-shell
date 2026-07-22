use crate::config::Config;
use nu_ansi_term::{Color, Style};
use reedline::ReedlineError;
use reedline::ReedlineErrorVariants::HistoryFeatureUnsupported;
use reedline::{History, SearchQuery};
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
pub struct Suggestions {
    current_suggestion: String,
    cwd_aware: bool,
    min_chars: usize,
    style: Style,
}

impl Suggestions {
    pub fn new(config: &Arc<Config>) -> Self {
        Self {
            current_suggestion: String::new(),
            cwd_aware: config.suggestions.cwd_aware,
            min_chars: config.suggestions.min_chars,
            style: Style::new().fg(Color::Fixed(config.suggestions.color)),
        }
    }

    fn search_history(&mut self, line: &str, cwd: &str, history: &dyn History) -> String {
        if line.chars().take(self.min_chars + 1).count() >= self.min_chars {
            return match self.cwd_aware {
                true => self.search_history_cwd(line, cwd, history),
                false => Self::search_history_global(line, history),
            };
        }

        String::new()
    }

    fn search_history_global(line: &str, history: &dyn History) -> String {
        history
            .search(SearchQuery::last_with_prefix(
                line.to_string(),
                history.session(),
            ))
            .unwrap_or_default()
            .first()
            .map_or_else(String::new, |entry| {
                entry
                    .command_line
                    .get(line.len()..)
                    .unwrap_or_default()
                    .to_string()
            })
    }

    fn search_history_cwd(&mut self, line: &str, cwd: &str, history: &dyn History) -> String {
        let line_string = line.to_string();

        let search_result = history.search(SearchQuery::last_with_prefix_and_cwd(
            line_string.clone(),
            cwd.to_string(),
            history.session(),
        ));

        let entry = match search_result {
            Ok(entries) => entries.into_iter().next(),
            Err(err) => {
                if let ReedlineError(HistoryFeatureUnsupported { .. }) = err {
                    self.cwd_aware = false;
                    history
                        .search(SearchQuery::last_with_prefix(
                            line_string,
                            history.session(),
                        ))
                        .ok()
                        .and_then(|entries| entries.into_iter().next())
                } else {
                    None
                }
            }
        };

        entry
            .and_then(|ent| ent.command_line.get(line.len()..).map(ToString::to_string))
            .unwrap_or_default()
    }
}

impl reedline::Hinter for Suggestions {
    fn handle(
        &mut self,
        line: &str,
        _pos: usize,
        history: &dyn History,
        use_ansi_coloring: bool,
        cwd: &str,
    ) -> String {
        self.current_suggestion = self.search_history(line, cwd, history);

        if use_ansi_coloring && !self.current_suggestion.is_empty() {
            self.style.paint(&self.current_suggestion).to_string()
        } else {
            self.current_suggestion.clone()
        }
    }

    fn complete_hint(&self) -> String {
        self.current_suggestion.clone()
    }

    fn next_hint_token(&self) -> String {
        let mut reached_content = false;
        let mut total_bytes = 0;

        for word in self.current_suggestion.split_word_bounds() {
            match (word.chars().all(char::is_whitespace), reached_content) {
                (_, true) => break,
                (true, false) => total_bytes += word.len(),
                (false, false) => {
                    reached_content = true;
                    total_bytes += word.len();
                }
            }
        }

        self.current_suggestion[..total_bytes].to_string()
    }
}
