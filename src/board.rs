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

    /// Removes the specified rows and compacts the board downward.
    /// Anchor cells in cleared rows are preserved: they are re-inserted at their
    /// adjusted position after compaction, then settled by `apply_anchor_gravity`.
    /// Call this after the line-clear animation finishes.
    pub fn remove_rows(&mut self, rows: &[usize]) {
        // 1. Collect anchor cells inside the cleared rows (before we touch anything).
        let mut saved_anchors: Vec<(usize, usize)> = Vec::new(); // (orig_row, col)
        for &r in rows {
            for c in 0..COLS {
                if matches!(self.cells[r][c], Some(PieceColor::Anchor)) {
                    saved_anchors.push((r, c));
                    self.cells[r][c] = None; // temporarily clear so compaction works cleanly
                }
            }
        }

        // 2. Standard compaction: skip the cleared rows.
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

        // 3. Re-insert each anchor at its adjusted row (shift down by however many
        //    cleared rows were at or below it, then let gravity settle the rest).
        for (orig_row, col) in saved_anchors {
            let shift = rows.iter().filter(|&&r| r <= orig_row).count();
            let new_row = orig_row.saturating_sub(shift);
            // Find the first empty row at or above new_row (handles collision when
            // two anchors in the same column are both cleared and land on the same slot).
            let target = (new_row..ROWS).find(|&r| self.cells[r][col].is_none());
            if let Some(r) = target {
                self.cells[r][col] = Some(PieceColor::Anchor);
            }
        }

        // 4. Settle any anchors that are now floating above empty cells.
        self.apply_anchor_gravity();
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
