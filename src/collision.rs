/// Collision detection: checks piece positions against the board and the other player's active piece.
use crate::board::Board;
use crate::piece::{self, Rotation, TSpinType, TetrominoKind};

/// Minimal position snapshot used for "other player" collision checks.
/// Contains only the fields needed to determine cell occupancy.
#[derive(Clone, Copy)]
pub struct PiecePos {
    pub kind: TetrominoKind,
    pub rotation: Rotation,
    pub col: i32,
    pub row: i32,
}

/// Returns the absolute cell positions for a piece at (col, row) with given kind and rotation.
pub fn absolute_cells(kind: TetrominoKind, rot: Rotation, col: i32, row: i32) -> [(i32, i32); 4] {
    piece::cells(kind, rot).map(|(dx, dy)| (col + dx, row + dy))
}

/// Check if a piece fits at (col, row) with given rotation.
/// `other` is the other player's active piece position (if any) to also avoid overlapping.
pub fn piece_fits(
    board: &Board,
    kind: TetrominoKind,
    rot: Rotation,
    col: i32,
    row: i32,
    other: Option<PiecePos>,
) -> bool {
    let cells = absolute_cells(kind, rot, col, row);
    let other_cells = other.map(|o| absolute_cells(o.kind, o.rotation, o.col, o.row));

    for (cx, cy) in cells {
        if !board.is_empty(cx, cy) {
            return false;
        }
        if let Some(ref oc) = other_cells {
            if oc.contains(&(cx, cy)) {
                return false;
            }
        }
    }
    true
}

/// Detects whether a T-piece locked via rotation constitutes a T-Spin.
/// Call this BEFORE writing the piece to the board (the piece cells must not yet be locked).
/// Returns Full T-Spin if ≥3 corners occupied and both front corners occupied;
/// Mini T-Spin if ≥3 corners but only 1 front corner; None otherwise.
pub fn check_t_spin(board: &Board, col: i32, row: i32, rot: Rotation) -> TSpinType {
    let corners = [
        (col - 1, row - 1),
        (col + 1, row - 1),
        (col - 1, row + 1),
        (col + 1, row + 1),
    ];
    let occupied = corners.iter().filter(|&&(cx, cy)| !board.is_empty(cx, cy)).count();
    if occupied < 3 {
        return TSpinType::None;
    }
    // Front corners depend on rotation (the side the stem of the T points toward).
    let front = match rot {
        Rotation::R0 => [(col - 1, row + 1), (col + 1, row + 1)], // stem up
        Rotation::R1 => [(col + 1, row + 1), (col + 1, row - 1)], // stem right
        Rotation::R2 => [(col - 1, row - 1), (col + 1, row - 1)], // stem down
        Rotation::R3 => [(col - 1, row + 1), (col - 1, row - 1)], // stem left
    };
    let front_occupied = front.iter().filter(|&&(cx, cy)| !board.is_empty(cx, cy)).count();
    if front_occupied == 2 {
        TSpinType::Full
    } else {
        TSpinType::Mini
    }
}

/// Try to rotate a piece using SRS wall kicks. Returns Some((new_col, new_row, new_rot)) on success.
pub fn try_rotate(
    board: &Board,
    kind: TetrominoKind,
    from_rot: Rotation,
    to_rot: Rotation,
    col: i32,
    row: i32,
    other: Option<PiecePos>,
) -> Option<(i32, i32, Rotation)> {
    let kicks = piece::kick_offsets(kind, from_rot, to_rot);
    for &(dx, dy) in kicks {
        let nc = col + dx;
        let nr = row + dy;
        if piece_fits(board, kind, to_rot, nc, nr, other) {
            return Some((nc, nr, to_rot));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    fn make_pos(kind: TetrominoKind, col: i32, row: i32) -> PiecePos {
        PiecePos { kind, rotation: Rotation::R0, col, row }
    }

    #[test]
    fn piece_fits_empty_board() {
        let board = Board::default();
        assert!(piece_fits(
            &board,
            TetrominoKind::T,
            Rotation::R0,
            9,
            20,
            None
        ));
    }

    #[test]
    fn piece_blocked_by_wall() {
        let board = Board::default();
        // I piece at R0 extends from col-1..col+2, so col=0 puts one cell at -1
        assert!(!piece_fits(
            &board,
            TetrominoKind::I,
            Rotation::R0,
            0,
            10,
            None
        ));
    }

    #[test]
    fn piece_blocked_by_other_player() {
        let board = Board::default();
        let other = make_pos(TetrominoKind::O, 5, 10);
        // O at (5,10) occupies (5,10),(6,10),(5,11),(6,11)
        // T at R0 centered at (5,10) occupies (4,10),(5,10),(6,10),(5,11) - overlap!
        assert!(!piece_fits(&board, TetrominoKind::T, Rotation::R0, 5, 10, Some(other)));
    }

    #[test]
    fn rotate_with_kick() {
        let mut board = Board::default();
        // Place wall on left side
        for row in 0..5 {
            board.set(0, row, crate::board::PieceColor::Player1(TetrominoKind::I));
        }
        // T at col=1, row=2, R0 -> R3: basic rotation puts cell at col-1=0 which is occupied
        // Wall kick should find a valid position
        let result = try_rotate(
            &board,
            TetrominoKind::T,
            Rotation::R0,
            Rotation::R3,
            1,
            2,
            None,
        );
        assert!(result.is_some());
    }

    #[test]
    fn rotate_blocked_by_other_player() {
        let board = Board::default();
        let mut board2 = Board::default();
        // Fill cols 3-7 rows 3-7 so T piece can't go anywhere
        for r in 3..=7i32 {
            for c in 3..=7i32 {
                board2.set(c, r, crate::board::PieceColor::Player1(TetrominoKind::I));
            }
        }
        // T at center of filled region should fail rotation
        let result = try_rotate(&board2, TetrominoKind::T, Rotation::R0, Rotation::R1, 5, 5, None);
        assert!(result.is_none(), "rotation should be blocked when all kicks are occupied");

        // Without the other piece the T at (5,5) R0->R1 succeeds (kick 0 = no offset, fits fine on empty board)
        let result_no_other = try_rotate(&board, TetrominoKind::T, Rotation::R0, Rotation::R1, 5, 5, None);
        assert!(result_no_other.is_some());
        // With board walls around AND other piece, confirm blocking still works
        let other = make_pos(TetrominoKind::O, 6, 5);
        let result_with_other = try_rotate(&board2, TetrominoKind::T, Rotation::R0, Rotation::R1, 5, 5, Some(other));
        assert!(result_with_other.is_none());
    }

    #[test]
    fn t_spin_full_detection() {
        let mut board = Board::default();
        let color = crate::board::PieceColor::Player1(TetrominoKind::I);
        // T at col=5, row=5, R1 (stem right). Front corners for R1: (col+1,row+1) and (col+1,row-1)
        // Occupy all 4 corners: (4,4),(6,4),(4,6),(6,6)
        board.set(4, 4, color);
        board.set(6, 4, color);
        board.set(4, 6, color);
        board.set(6, 6, color);
        // All 4 corners occupied → ≥3 and both front corners (6,6) and (6,4) occupied → Full
        assert_eq!(check_t_spin(&board, 5, 5, Rotation::R1), TSpinType::Full);
    }

    #[test]
    fn t_spin_mini_detection() {
        let mut board = Board::default();
        let color = crate::board::PieceColor::Player1(TetrominoKind::I);
        // T at col=5, row=5, R1. Front corners: (6,6) and (6,4).
        // Occupy 3 corners but only 1 front corner → Mini
        board.set(4, 4, color); // back-left
        board.set(4, 6, color); // back-right (back for R1)
        board.set(6, 4, color); // front-bottom (1 front corner)
        // 3 corners occupied, only 1 front → Mini
        assert_eq!(check_t_spin(&board, 5, 5, Rotation::R1), TSpinType::Mini);
    }

    #[test]
    fn t_spin_none_detection() {
        let mut board = Board::default();
        let color = crate::board::PieceColor::Player1(TetrominoKind::I);
        // Only 2 corners occupied → None
        board.set(4, 4, color);
        board.set(6, 4, color);
        assert_eq!(check_t_spin(&board, 5, 5, Rotation::R0), TSpinType::None);
    }

    #[test]
    fn piece_on_floor() {
        let board = Board::default();
        // T at R0, row=0: cells at row 0 and row 1 - should fit
        assert!(piece_fits(
            &board,
            TetrominoKind::T,
            Rotation::R0,
            9,
            0,
            None,
        ));
        // T at R2, row=0: has cell at row -1 - should not fit
        assert!(!piece_fits(
            &board,
            TetrominoKind::T,
            Rotation::R2,
            9,
            0,
            None,
        ));
    }
}
