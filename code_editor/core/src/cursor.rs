use crate::{text, text::DeltaLen};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Cursor {
    pub caret: Position,
    pub anchor: Position,
}

impl Cursor {
    pub fn is_empty(self) -> bool {
        self.caret == self.anchor
    }

    pub fn start(self) -> Position {
        self.caret.min(self.anchor)
    }

    pub fn end(self) -> Position {
        self.caret.max(self.anchor)
    }

    pub fn apply_delta(self, delta_len: DeltaLen) -> Self {
        Self {
            caret: self.caret.apply_delta(delta_len),
            anchor: self.anchor.apply_delta(delta_len),
        }
    }

    pub fn merge(mut self, mut other: Self) -> Option<Self> {
        use std::{cmp::Ordering, mem};

        if self.start() > other.start() {
            mem::swap(&mut self, &mut other);
        }
        match (self.is_empty(), other.is_empty()) {
            (true, true) if self.caret.position == other.caret.position => Some(self),
            (false, true) if other.caret.position <= self.end().position => Some(self),
            (true, false) if self.caret.position == other.start().position => Some(other),
            (false, false) if self.end().position > other.start().position => {
                Some(match self.caret.cmp(&self.anchor) {
                    Ordering::Less => Self {
                        caret: self.caret.min(other.caret),
                        anchor: self.anchor.max(other.anchor),
                    },
                    Ordering::Greater => Self {
                        caret: self.caret.max(other.caret),
                        anchor: self.anchor.min(other.anchor),
                    },
                    Ordering::Equal => unreachable!(),
                })
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub position: text::Position,
    pub affinity: Affinity,
}

impl Position {
    pub fn is_right_before(self, position: text::Position) -> bool {
        self.position == position && self.affinity == Affinity::Before
    }

    pub fn is_right_after(self, position: text::Position) -> bool {
        self.position == position && self.affinity == Affinity::After
    }

    pub fn apply_delta(self, delta_len: DeltaLen) -> Self {
        Self {
            position: self.position.apply_delta(delta_len),
            affinity: self.affinity,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Affinity {
    Before,
    After,
}

impl Default for Affinity {
    fn default() -> Self {
        Self::Before
    }
}
