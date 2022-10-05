use crate::SearchDirection;
use crate::highlighting;
use std::cmp;
use tui::symbols::line::THICK_BOTTOM_RIGHT;
use unicode_segmentation::UnicodeSegmentation;
use termion::color;

#[derive(Default)]
pub struct Row {
    string: String,
    highlighting: Vec<highlighting::Type>,
    len: usize,
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
        let mut current_highlighting = &highlighting::Type::None;

        #[allow(clippy::integer_arithmetic)]
        for (index, grapheme) in self.string[..]
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end-start)
        {
            if let Some(c) = grapheme.chars().next() {
                let highlighting_type = self
                    .highlighting
                    .get(index)
                    .unwrap_or(&highlighting::Type::None);
                if highlighting_type != current_highlighting {
                    current_highlighting = highlighting_type;
                    let start_highlight =
                        format!("{}", termion::color::Fg(highlighting_type.to_color()));
                    result.push_str(&start_highlight[..]);
                }
                if c == '\t' {
                    result.push_str("    ");
                } else {
                    result.push(c);
                }
            }
        }

        let end_highlight = format!("{}", termion::color::Fg(color::Reset));
        result.push_str(&end_highlight[..]);
        result
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.string.is_empty()
    }

    fn update_len(&mut self) {
        self.len = self.string[..].graphemes(true).count();
    }

    pub fn insert(&mut self,  at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
        } else {
            let mut result: String = self.string[..].graphemes(true).take(at).collect();
            let remainder: String = self.string[..].graphemes(true).skip(at).collect();
            result.push(c);
            result.push_str(&remainder);
            self.string = result;
        }
        self.update_len();
    }

    #[allow(clippy::integer_arithmetic)]
    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        } else {
            let mut result: String = self.string[..].graphemes(true).take(at).collect();
            let remainder: String = self.string[..].graphemes(true).skip(at+1).collect();
            result.push_str(&remainder);
            self.string = result;
        }
        self.update_len();
    }

    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.update_len();
    }

    pub fn highlight(&mut self, word: Option<&str>) {
        let mut highlighting = Vec::new();
        let chars: Vec<char> = self.string.chars().collect();
        let mut matches = Vec::new();
        let mut search_index = 0;

        if let Some(word) = word {
            while let Some(search_match) = self.find(word, search_index, SearchDirection::Forward) {
                matches.push(search_match);
                if let Some(next_index) = search_match.checked_add(word[..].graphemes(true).count())
                    {
                        search_index = next_index;
                    } else {
                        break;
                    }
            }
        }

        let mut index = 0;
        while let Some(c) = chars.get(index) {
            if let Some(word) = word {
                if matches.contains(&index) {
                    for _ in word[..].graphemes(true) {
                        index += 1;
                        highlighting.push(highlighting::Type::Search);
                    }
                    continue;
                }
            }
            if c.is_ascii_digit() {
                highlighting.push(highlighting::Type::Number);
            } else {
                highlighting.push(highlighting::Type::None);
            }
            index += 1;
        }
        self.highlighting = highlighting;
    }

    pub fn split(&mut self, at: usize) -> Self {
        let beginning: String = self.string[..].graphemes(true).take(at).collect();
        let remainder: String = self.string[..].graphemes(true).skip(at).collect();
        self.string = beginning;
        self.update_len();
        Self::from(&remainder[..])
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if at > self.len() || query.is_empty() {
            return None;
        }
        let start = match direction {
            SearchDirection::Forward => at,
            _ => 0,
        };

        let end = match direction {
            SearchDirection::Forward => self.len,
            _ => at,
        };

        #[allow(clippy::integer_arithmetic)]
        let substring: String = self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();

        let matching_byte_index = match direction {
            SearchDirection::Forward => substring.find(query),
            _ => substring.rfind(query),
        };
        if let Some(matching_byte_index) = matching_byte_index {
            for (grapheme_index, (byte_index, _)) in
                substring[..].grapheme_indices(true).enumerate()
            {
                if matching_byte_index == byte_index {
                    #[allow(clippy::integer_arithmetic)]
                    return Some(start + grapheme_index);
                }
            }
        }
        None
    }
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        let mut row = Self {
            string: String::from(slice),
            highlighting: Vec::new(),
            len: 0,
        };
        row.update_len();

        row
    }
}
