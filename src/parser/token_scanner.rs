use std::iter::Peekable;
use std::str::Chars;

pub struct TokenScanner<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> TokenScanner<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    pub fn next_char(&mut self) -> Option<char> {
        self.chars.next()
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
