use core::iter;

enum LineEnd {
    Hard(usize),
    Soft(usize),
}

impl LineEnd {
    fn index(&self) -> usize {
        match *self {
            Self::Hard(index) | Self::Soft(index) => index,
        }
    }
}

pub struct TerminalState {
    text: String,
    line_ends: Vec<LineEnd>,
    max_line_len: usize,
}

impl TerminalState {
    pub fn new(max_line_len: usize) -> Self {
        Self {
            text: String::new(),
            line_ends: Vec::new(),
            max_line_len,
        }
    }

    pub fn write(&mut self, s: &str) {
        if let Some(ch) = self.text.pop() {
            if ch != '\0' {
                self.text.push(ch);
            }
        }

        self.text.reserve(s.len());

        let mut line_char_count = if let Some(prev_end) = self.line_ends.last() {
            self.text[prev_end.index()..].chars().count()
        } else {
            self.text.chars().count()
        };

        for ch in s.chars() {
            if ch == '\n' {
                self.text.push('\0');
                line_char_count = 0;
                self.line_ends.push(LineEnd::Hard(self.text.len()));
            } else {
                assert!(line_char_count < self.max_line_len);

                self.text.push(ch);
                line_char_count += 1;

                if line_char_count == self.max_line_len {
                    self.line_ends.push(LineEnd::Soft(self.text.len()));
                    line_char_count = 0;
                }
            }
        }

        self.text.push('\0');
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn rewrap(&mut self, max_line_len: usize) {
        let ends = self
            .line_ends
            .drain(..)
            .filter_map(|end| if let LineEnd::Hard(end) = end { Some(end) } else { None })
            .collect::<Vec<_>>();

        self.max_line_len = max_line_len;

        fn soft_wrap(line_ends: &mut Vec<LineEnd>, text: &str, max_line_len: usize, mut prev_end: usize, end: usize) {
            let mut s = &text[prev_end..end];
            while s.chars().count() > max_line_len {
                prev_end += s.chars().take(max_line_len).map(|ch| ch.len_utf8()).sum::<usize>();
                s = &text[prev_end..end];
                line_ends.push(LineEnd::Soft(prev_end));
            }
        }

        let mut prev_end = 0;
        for end in ends {
            soft_wrap(&mut self.line_ends, &self.text, max_line_len, prev_end, end);
            self.line_ends.push(LineEnd::Hard(end));
            prev_end = end;
        }

        soft_wrap(&mut self.line_ends, &self.text, max_line_len, prev_end, self.text.len());
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        Iterator::zip(
            iter::once(0).chain(self.line_ends.iter().map(|end| end.index())),
            self.line_ends
                .iter()
                .map(|end| end.index())
                .chain(iter::once(self.text.len())),
        )
        .map(move |(prev_end, end)| &self.text[prev_end..end])
    }
}

#[cfg(test)]
mod test {
    use crate::state::TerminalState;

    fn check<F, G>(mut f: F, mut g: G)
    where
        F: FnMut(&mut TerminalState),
        G: FnMut(TerminalState),
    {
        let mut state = TerminalState::new(10);
        f(&mut state);
        g(state);

        let mut state = TerminalState::new(20);
        f(&mut state);
        state.rewrap(10);
        g(state);

        let mut state = TerminalState::new(5);
        f(&mut state);
        state.rewrap(10);
        g(state);
    }

    #[test]
    fn can_write_partial_line() {
        check(
            |state| state.write("hello💣"),
            |state| {
                assert_eq!(&state.text, "hello💣\0");
                assert_eq!(state.lines().collect::<Vec<_>>(), vec!["hello💣\0"]);
            },
        );
    }

    #[test]
    fn can_continue_partial_line() {
        check(
            |state| {
                state.write("he");
                assert_eq!(&state.text, "he\0");
                assert_eq!(state.lines().collect::<Vec<_>>(), vec!["he\0"]);

                state.write("llo💣");
            },
            |state| {
                assert_eq!(&state.text, "hello💣\0");
                assert_eq!(state.lines().collect::<Vec<_>>(), vec!["hello💣\0"]);
            },
        );
    }

    #[test]
    fn can_write_full_line() {
        check(
            |state| state.write("hello💣\n"),
            |state| {
                assert_eq!(&state.text, "hello💣\0\0");
                assert_eq!(state.lines().collect::<Vec<_>>(), vec!["hello💣\0", "\0"]);
            },
        );
    }

    #[test]
    fn can_write_full_line_partial_line() {
        check(
            |state| state.write("hello💣\nworld"),
            |state| {
                assert_eq!(&state.text, "hello💣\0world\0");
                assert_eq!(state.lines().collect::<Vec<_>>(), vec!["hello💣\0", "world\0"]);
            },
        );
    }

    #[test]
    fn can_write_full_line_partial_line_continue() {
        check(
            |state| {
                state.write("hello💣\nwo");
                assert_eq!(&state.text, "hello💣\0wo\0");

                if state.max_line_len > 5 {
                    assert_eq!(state.lines().collect::<Vec<_>>(), vec!["hello💣\0", "wo\0"]);
                } else {
                    assert_eq!(state.max_line_len, 5);
                    assert_eq!(state.lines().collect::<Vec<_>>(), vec!["hello", "💣\0", "wo\0"]);
                }

                state.write("rld");
            },
            |state| {
                assert_eq!(&state.text, "hello💣\0world\0");
                assert_eq!(state.lines().collect::<Vec<_>>(), vec!["hello💣\0", "world\0"]);
            },
        );
    }

    #[test]
    fn can_wrap_long_line() {
        check(
            |state| {
                state.write("💣123456789abcdef");
                assert_eq!(&state.text, "💣123456789abcdef\0");

                match state.max_line_len {
                    20 => {
                        assert_eq!(state.lines().collect::<Vec<_>>(), vec!["💣123456789abcdef\0"]);
                    }
                    10 => {
                        assert_eq!(state.lines().collect::<Vec<_>>(), vec!["💣123456789", "abcdef\0"]);
                    }
                    5 => {
                        assert_eq!(
                            state.lines().collect::<Vec<_>>(),
                            vec!["💣1234", "56789", "abcde", "f\0"]
                        );
                    }
                    _ => panic!(),
                }

                state.write("123456");
            },
            |state| {
                assert_eq!(&state.text, "💣123456789abcdef123456\0");
                assert_eq!(
                    state.lines().collect::<Vec<_>>(),
                    vec!["💣123456789", "abcdef1234", "56\0"]
                );
            },
        );
    }
}
