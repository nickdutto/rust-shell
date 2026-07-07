use std::iter::Peekable;
use std::str::CharIndices;

pub struct TokenScanner<'a> {
    chars: Peekable<CharIndices<'a>>,
    current_index: usize,
}

impl<'a> TokenScanner<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.char_indices().peekable(),
            current_index: 0,
        }
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn peek(&mut self) -> Option<&char> {
        if let Some((_, ch)) = self.chars.peek() {
            Some(ch)
        } else {
            None
        }
    }

    pub fn next_char(&mut self) -> Option<char> {
        if let Some((idx, ch)) = self.chars.next() {
            self.current_index = idx + ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }

    pub fn next_if_matches(&mut self, expected: char) -> bool {
        if self.peek() == Some(&expected) {
            self.next_char();
            true
        } else {
            false
        }
    }
}
