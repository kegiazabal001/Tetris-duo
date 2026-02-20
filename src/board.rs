/// Board grid: 18 columns wide, 22 rows tall (20 visible + 2 spawn rows).
use bevy::prelude::*;

use crate::piece::TetrominoKind;

pub const COLS: usize = 18;
pub const ROWS: usize = 22;
pub const VISIBLE_ROWS: usize = 20;

/// Color stored per locked cell: player ownership + piece kind (for per-piece coloring).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceColor {
    Player1(TetrominoKind),
    Player2(TetrominoKind),
}

impl PieceColor {
    pub fn kind(self) -> TetrominoKind {
        match self {
            PieceColor::Player1(k) | PieceColor::Player2(k) => k,
        }
    }
    pub fn is_player1(self) -> bool {
        matches!(self, PieceColor::Player1(_))
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
    /// Call this after the line-clear animation finishes.
    pub fn remove_rows(&mut self, rows: &[usize]) {
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
