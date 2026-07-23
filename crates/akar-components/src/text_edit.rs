#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextEditState {
    pub cursor: usize,
    pub anchor: usize,
}

impl TextEditState {
    pub fn normalize(&mut self, value: &str) {
        self.cursor = normalize_position(value, self.cursor);
        self.anchor = normalize_position(value, self.anchor);
    }

    pub fn selection(&self) -> std::ops::Range<usize> {
        self.anchor.min(self.cursor)..self.anchor.max(self.cursor)
    }

    pub fn has_selection(&self) -> bool {
        self.cursor != self.anchor
    }

    pub fn select_all(&mut self, value: &str) {
        self.anchor = 0;
        self.cursor = value.len();
    }

    pub fn collapse_to_start(&mut self) {
        self.cursor = self.selection().start;
        self.anchor = self.cursor;
    }

    pub fn collapse_to_end(&mut self) {
        self.cursor = self.selection().end;
        self.anchor = self.cursor;
    }
}

pub fn normalize_position(value: &str, position: usize) -> usize {
    let mut position = position.min(value.len());
    while position > 0 && !value.is_char_boundary(position) {
        position -= 1;
    }
    position
}

pub fn replace_selection(value: &mut String, state: &mut TextEditState, replacement: &str) -> bool {
    state.normalize(value);
    let range = state.selection();
    let changed = !range.is_empty() || !replacement.is_empty();
    value.replace_range(range.clone(), replacement);
    let position = range.start + replacement.len();
    state.cursor = position;
    state.anchor = position;
    changed
}

pub fn delete_selection(value: &mut String, state: &mut TextEditState) -> bool {
    state.normalize(value);
    let range = state.selection();
    if range.is_empty() {
        return false;
    }
    value.replace_range(range.clone(), "");
    state.cursor = range.start;
    state.anchor = range.start;
    true
}

pub fn normalize_paste(text: &str, multiline: bool) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    if multiline {
        normalized
    } else {
        normalized.replace('\n', " ")
    }
}

pub fn previous_boundary(value: &str, position: usize) -> usize {
    let position = normalize_position(value, position);
    value[..position]
        .char_indices()
        .next_back()
        .map_or(0, |(index, _)| index)
}

pub fn next_boundary(value: &str, position: usize) -> usize {
    let position = normalize_position(value, position);
    value[position..]
        .chars()
        .next()
        .map_or(value.len(), |character| position + character.len_utf8())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_invalid_utf8_positions() {
        let value = "aé🙂";
        let mut state = TextEditState {
            cursor: 3,
            anchor: 99,
        };
        state.normalize(value);
        assert_eq!(
            state,
            TextEditState {
                cursor: 3,
                anchor: value.len()
            }
        );
    }

    #[test]
    fn replaces_unicode_and_newline_selection() {
        let mut value = "aé\n🙂z".to_owned();
        let mut state = TextEditState {
            cursor: 7,
            anchor: 1,
        };
        assert!(replace_selection(&mut value, &mut state, "x"));
        assert_eq!(value, "ax🙂z");
        assert_eq!(
            state,
            TextEditState {
                cursor: 2,
                anchor: 2
            }
        );
    }

    #[test]
    fn paste_normalization_is_mode_specific() {
        assert_eq!(normalize_paste("a\r\nb\rc", false), "a b c");
        assert_eq!(normalize_paste("a\r\nb\rc", true), "a\nb\nc");
    }
}
