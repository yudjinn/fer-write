use crate::SearchDirection;
use crate::Row;
use crate::Position;
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub filename: Option<String>,
    dirty: bool,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for value in contents.lines() {
            let mut row = Row::from(value);
            row.highlight(None);
            rows.push(row);
        }

        Ok(Self {
            rows,
            filename: Some(filename.to_string()),
            dirty: false,
        })
    }

    pub fn highlight(&mut self, word: Option<&str>) {
        for row in &mut self.rows {
            row.highlight(word)
        }
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(file_name) = &self.filename {
            let mut file = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }
        self.dirty = false;
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn insert_newline(&mut self, at: &Position) {
        if at.y > self.len() {
            return;
        }
        if at.y == self.len() {
            self.rows.push(Row::default());
        }

        let current_row = &mut self.rows[at.y];
        let mut new_row = current_row.split(at.x);
        current_row.highlight(None);
        new_row.highlight(None);
        #[allow(clippy::integer_arithmetic)]
        self.rows.insert(at.y + 1, new_row);
    }

    pub fn insert(&mut self , at: &Position, c: char) {
        self.dirty = true;
        match c {
            '\n' => {
                self.insert_newline(at);
            },
            '\t' => {
                for _ in 0..4 {
                    self.insert(&at, ' ');
                }
            },
            _ => {
                if at.y == self.len() {
                    let mut row = Row::default();
                    row.insert(0, c);
                    row.highlight(None);
                    self.rows.push(row);
                } else if at.y < self.len() {
                    let row = self.rows.get_mut(at.y).unwrap();
                    row.insert(at.x, c);
                    row.highlight(None);
                }
            }
        };
    }
    #[allow(clippy::integer_arithmetic)]
    pub fn delete(&mut self, at: &Position) {
        self.dirty = true;
        let len = self.len();
        if at.y == len {
            return;
        }

        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y +1 < len {
            let next_row = self.rows.remove(at.y + 1);
            let row = self.rows.get_mut(at.y).unwrap();
            row.append(&next_row);
            row.highlight(None);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x);
            row.highlight(None);
        }
    }

    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if at.y >= self.rows.len() {
            return None;
        }
        let mut position = Position {x: at.x, y: at.y};

        let start = match direction {
            SearchDirection::Forward => at.y,
            _ => 0,
        };

        let end = match direction {
            SearchDirection::Forward => self.rows.len(),
            _ => at.y.saturating_add(1),
        };

        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(&query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }
                match direction {
                    SearchDirection::Forward => {
                        position.y = position.y.saturating_add(1);
                        position.x = 0;
                    },
                    _ => {
                        position.y = position.y.saturating_sub(1);
                        position.x = self.rows[position.y].len();
                    }
                }
            } else {
                return None;
            }
        }
        None
    }
}
