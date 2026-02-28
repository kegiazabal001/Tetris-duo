/// Board grid: 16 columns wide, 22 rows tall (20 visible + 2 spawn rows).
use bevy::prelude::*;

use crate::piece::TetrominoKind;

pub const COLS: usize = 16;
pub const ROWS: usize = 22;
pub const VISIBLE_ROWS: usize = 20;

/// Color stored per locked cell: player ownership + piece kind (for per-piece coloring).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceColor {
    Player1(TetrominoKind),
    Player2(TetrominoKind),
    /// Anchor piece — stays in the board permanently (survives line clears).
    Anchor,
}

impl PieceColor {
    pub fn kind(self) -> TetrominoKind {
        match self {
            PieceColor::Player1(k) | PieceColor::Player2(k) => k,
            PieceColor::Anchor => TetrominoKind::I, // fallback, unused visually
        }
    }
    pub fn is_player1(self) -> bool {
        matches!(self, PieceColor::Player1(_))
    }
    pub fn is_anchor(self) -> bool {
        matches!(self, PieceColor::Anchor)
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Board {
    /// cells[row][col], row 0 = bottom.
    pub cells: [[Option<PieceColor>; COLS]; ROWS],
}

impl Default for Board {
    fn default() -> Self {
        Self {
            cells: [[None; COLS]; ROWS],
        }
    }
}

impl Board {
    pub fn get(&self, col: i32, row: i32) -> Option<PieceColor> {
        if col < 0 || col >= COLS as i32 || row < 0 || row >= ROWS as i32 {
            return None;
        }
        self.cells[row as usize][col as usize]
    }

    pub fn is_empty(&self, col: i32, row: i32) -> bool {
        if col < 0 || col >= COLS as i32 || row < 0 || row >= ROWS as i32 {
            return false;
        }
        self.cells[row as usize][col as usize].is_none()
    }

    pub fn set(&mut self, col: i32, row: i32, color: PieceColor) {
        if col >= 0 && col < COLS as i32 && row >= 0 && row < ROWS as i32 {
            self.cells[row as usize][col as usize] = Some(color);
        }
    }

    /// Clears all full rows immediately. Returns the number of rows cleared.
    pub fn clear_full_rows(&mut self) -> u32 {
        let mut cleared = 0u32;
        let mut write = 0usize;
        for read in 0..ROWS {
            if self.cells[read].iter().all(|c| c.is_some()) {
                cleared += 1;
            } else {
                if write != read {
                    self.cells[write] = self.cells[read];
                }
                write += 1;
            }
        }
        for row in write..ROWS {
            self.cells[row] = [None; COLS];
        }
        cleared
    }

    /// Returns the indices of rows that are currently full (without removing them).
    /// Used to start the line-clear flash animation before compacting the board.
    pub fn detect_full_rows(&self) -> Vec<usize> {
        (0..ROWS)
            .filter(|&r| self.cells[r].iter().all(|c| c.is_some()))
            .collect()
    }

    /// Removes the specified rows and compacts the board.
    /// `gravity_dir`: +1 = normal (compact downward, empty rows at top);
    ///                -1 = inverted (compact upward, empty rows at bottom).
    /// Anchor cells in cleared rows are preserved and re-settled after compaction.
    /// Call this after the line-clear animation finishes.
    pub fn remove_rows(&mut self, rows: &[usize], gravity_dir: i8) {
        // 1. Collect anchor cells inside the cleared rows.
        let mut saved_anchors: Vec<(usize, usize)> = Vec::new(); // (orig_row, col)
        for &r in rows {
            for c in 0..COLS {
                if matches!(self.cells[r][c], Some(PieceColor::Anchor)) {
                    saved_anchors.push((r, c));
                    self.cells[r][c] = None;
                }
            }
        }

        if gravity_dir >= 0 {
            // Normal: compact downward — non-cleared rows shift toward row 0.
            let mut write = 0usize;
            for read in 0..ROWS {
                if !rows.contains(&read) {
                    if write != read {
                        self.cells[write] = self.cells[read];
                    }
                    write += 1;
                }
            }
            for row in write..ROWS {
                self.cells[row] = [None; COLS];
            }

            // Re-insert anchors: shift down by number of cleared rows at or below.
            for (orig_row, col) in saved_anchors {
                let shift = rows.iter().filter(|&&r| r <= orig_row).count();
                let new_row = orig_row.saturating_sub(shift);
                let target = (new_row..ROWS).find(|&r| self.cells[r][col].is_none());
                if let Some(r) = target {
                    self.cells[r][col] = Some(PieceColor::Anchor);
                }
            }
            self.apply_anchor_gravity();
        } else {
            // Inverted: compact upward — non-cleared rows shift toward row ROWS-1.
            let mut write = ROWS - 1;
            let mut wrote_any = false;
            for read in (0..ROWS).rev() {
                if !rows.contains(&read) {
                    if wrote_any || write != read {
                        self.cells[write] = self.cells[read];
                    }
                    wrote_any = true;
                    if write > 0 { write -= 1; } else { break; }
                }
            }
            // Clear remaining lower rows that are now empty.
            let clear_up_to = if wrote_any { write } else { ROWS - 1 };
            for row in 0..=clear_up_to {
                self.cells[row] = [None; COLS];
            }

            // Re-insert anchors: shift up by number of cleared rows at or above.
            for (orig_row, col) in saved_anchors {
                let shift = rows.iter().filter(|&&r| r >= orig_row).count();
                let new_row = (orig_row + shift).min(ROWS - 1);
                // Find first empty row at or below new_row (searching downward).
                let target = (0..=new_row).rev().find(|&r| self.cells[r][col].is_none());
                if let Some(r) = target {
                    self.cells[r][col] = Some(PieceColor::Anchor);
                }
            }
            self.apply_anchor_gravity_up();
        }
    }

    /// Drops every anchor cell one step at a time until all anchors rest on
    /// a filled cell or the board floor.
    pub fn apply_anchor_gravity(&mut self) {
        loop {
            let mut moved = false;
            // Iterate from row 1 upward so an anchor can fall multiple rows per call.
            for row in 1..ROWS {
                for col in 0..COLS {
                    if matches!(self.cells[row][col], Some(PieceColor::Anchor))
                        && self.cells[row - 1][col].is_none()
                    {
                        self.cells[row - 1][col] = Some(PieceColor::Anchor);
                        self.cells[row][col] = None;
                        moved = true;
                    }
                }
            }
            if !moved {
                break;
            }
        }
    }

    /// Raises every anchor cell one step at a time until all anchors rest on
    /// a filled cell or the board ceiling. Used during inverted gravity.
    fn apply_anchor_gravity_up(&mut self) {
        loop {
            let mut moved = false;
            // Iterate top-down so an anchor can rise multiple rows per call.
            for row in (0..ROWS - 1).rev() {
                for col in 0..COLS {
                    if matches!(self.cells[row][col], Some(PieceColor::Anchor))
                        && self.cells[row + 1][col].is_none()
                    {
                        self.cells[row + 1][col] = Some(PieceColor::Anchor);
                        self.cells[row][col] = None;
                        moved = true;
                    }
                }
            }
            if !moved {
                break;
            }
        }
    }

    /// Moves every non-empty cell one row in `dir` (+1 = up toward ROWS-1,
    /// -1 = down toward 0). Returns `true` if at least one cell moved.
    /// Used for the FLIP! migration animation.
    pub fn apply_gravity_step(&mut self, dir: i8) -> bool {
        let mut moved = false;
        if dir > 0 {
            // Move upward: iterate top-down so cells don't block each other.
            for row in (0..ROWS - 1).rev() {
                for col in 0..COLS {
                    if self.cells[row][col].is_some() && self.cells[row + 1][col].is_none() {
                        self.cells[row + 1][col] = self.cells[row][col];
                        self.cells[row][col] = None;
                        moved = true;
                    }
                }
            }
        } else {
            // Move downward: iterate bottom-up so cells don't block each other.
            for row in 1..ROWS {
                for col in 0..COLS {
                    if self.cells[row][col].is_some() && self.cells[row - 1][col].is_none() {
                        self.cells[row - 1][col] = self.cells[row][col];
                        self.cells[row][col] = None;
                        moved = true;
                    }
                }
            }
        }
        moved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board() {
        let board = Board::default();
        for row in 0..ROWS {
            for col in 0..COLS {
                assert!(board.cells[row][col].is_none());
            }
        }
    }

    #[test]
    fn set_and_get() {
        let mut board = Board::default();
        board.set(5, 3, PieceColor::Player1(TetrominoKind::T));
        assert_eq!(board.get(5, 3), Some(PieceColor::Player1(TetrominoKind::T)));
        assert!(board.is_empty(5, 4));
    }

    #[test]
    fn out_of_bounds() {
        let board = Board::default();
        assert!(!board.is_empty(-1, 0));
        assert!(!board.is_empty(0, -1));
        assert!(!board.is_empty(COLS as i32, 0));
        assert!(!board.is_empty(0, ROWS as i32));
    }

    #[test]
    fn clear_one_full_row() {
        let mut board = Board::default();
        for col in 0..COLS {
            board.set(col as i32, 0, PieceColor::Player1(TetrominoKind::I));
        }
        board.set(3, 1, PieceColor::Player2(TetrominoKind::T));

        let cleared = board.clear_full_rows();
        assert_eq!(cleared, 1);
        assert_eq!(board.get(3, 0), Some(PieceColor::Player2(TetrominoKind::T)));
        assert!(board.is_empty(0, 1));
    }

    #[test]
    fn clear_multiple_rows() {
        let mut board = Board::default();
        for row in 0..3 {
            for col in 0..COLS {
                board.set(col as i32, row, PieceColor::Player1(TetrominoKind::S));
            }
        }
        board.set(0, 3, PieceColor::Player2(TetrominoKind::Z));

        let cleared = board.clear_full_rows();
        assert_eq!(cleared, 3);
        assert_eq!(board.get(0, 0), Some(PieceColor::Player2(TetrominoKind::Z)));
    }
}
