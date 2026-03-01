/// Tetromino definitions, SRS rotation tables, wall kick data, and T-Spin type.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TSpinType {
    #[default]
    None,
    Mini,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TetrominoKind {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl TetrominoKind {
    pub const ALL: [TetrominoKind; 7] = [
        TetrominoKind::I,
        TetrominoKind::O,
        TetrominoKind::T,
        TetrominoKind::S,
        TetrominoKind::Z,
        TetrominoKind::J,
        TetrominoKind::L,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rotation {
    R0,
    R1, // CW from spawn
    R2, // 180
    R3, // CCW from spawn
}

impl Rotation {
    pub fn cw(self) -> Self {
        match self {
            Rotation::R0 => Rotation::R1,
            Rotation::R1 => Rotation::R2,
            Rotation::R2 => Rotation::R3,
            Rotation::R3 => Rotation::R0,
        }
    }

    pub fn ccw(self) -> Self {
        match self {
            Rotation::R0 => Rotation::R3,
            Rotation::R1 => Rotation::R0,
            Rotation::R2 => Rotation::R1,
            Rotation::R3 => Rotation::R2,
        }
    }
}

/// Returns the 4 cell offsets (col, row) for a given piece and rotation.
/// Origin is the rotation center; row increases upward in game logic.
pub fn cells(kind: TetrominoKind, rot: Rotation) -> [(i32, i32); 4] {
    use Rotation::*;
    use TetrominoKind::*;
    match (kind, rot) {
        // I piece
        (I, R0) => [(-1, 0), (0, 0), (1, 0), (2, 0)],
        (I, R1) => [(0, 1), (0, 0), (0, -1), (0, -2)],
        (I, R2) => [(-1, -1), (0, -1), (1, -1), (2, -1)],
        (I, R3) => [(1, 1), (1, 0), (1, -1), (1, -2)],
        // O piece
        (O, _) => [(0, 0), (1, 0), (0, 1), (1, 1)],
        // T piece
        (T, R0) => [(-1, 0), (0, 0), (1, 0), (0, 1)],
        (T, R1) => [(0, 1), (0, 0), (0, -1), (1, 0)],
        (T, R2) => [(-1, 0), (0, 0), (1, 0), (0, -1)],
        (T, R3) => [(0, 1), (0, 0), (0, -1), (-1, 0)],
        // S piece
        (S, R0) => [(-1, 0), (0, 0), (0, 1), (1, 1)],
        (S, R1) => [(0, 1), (0, 0), (1, 0), (1, -1)],
        (S, R2) => [(-1, -1), (0, -1), (0, 0), (1, 0)],
        (S, R3) => [(-1, 1), (-1, 0), (0, 0), (0, -1)],
        // Z piece
        (Z, R0) => [(-1, 1), (0, 1), (0, 0), (1, 0)],
        (Z, R1) => [(1, 1), (1, 0), (0, 0), (0, -1)],
        (Z, R2) => [(-1, 0), (0, 0), (0, -1), (1, -1)],
        (Z, R3) => [(0, 1), (0, 0), (-1, 0), (-1, -1)],
        // J piece
        (J, R0) => [(-1, 1), (-1, 0), (0, 0), (1, 0)],
        (J, R1) => [(0, 1), (1, 1), (0, 0), (0, -1)],
        (J, R2) => [(-1, 0), (0, 0), (1, 0), (1, -1)],
        (J, R3) => [(0, 1), (0, 0), (-1, -1), (0, -1)],
        // L piece
        (L, R0) => [(-1, 0), (0, 0), (1, 0), (1, 1)],
        (L, R1) => [(0, 1), (0, 0), (0, -1), (1, -1)],
        (L, R2) => [(-1, -1), (-1, 0), (0, 0), (1, 0)],
        (L, R3) => [(-1, 1), (0, 1), (0, 0), (0, -1)],
    }
}

/// SRS wall kick offsets. Returns up to 5 (dx, dy) offsets to try.
/// `from` -> `to` rotation transition.
pub fn kick_offsets(kind: TetrominoKind, from: Rotation, to: Rotation) -> &'static [(i32, i32)] {
    use Rotation::*;
    if kind == TetrominoKind::O {
        return &[(0, 0)];
    }
    if kind == TetrominoKind::I {
        return match (from, to) {
            (R0, R1) => &[(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
            (R1, R0) => &[(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
            (R1, R2) => &[(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
            (R2, R1) => &[(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
            (R2, R3) => &[(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
            (R3, R2) => &[(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
            (R3, R0) => &[(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
            (R0, R3) => &[(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
            _ => &[(0, 0)],
        };
    }
    // JLSTZ
    match (from, to) {
        (R0, R1) => &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        (R1, R0) => &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        (R1, R2) => &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        (R2, R1) => &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        (R2, R3) => &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        (R3, R2) => &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        (R3, R0) => &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        (R0, R3) => &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        _ => &[(0, 0)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_pieces_have_4_cells() {
        for kind in TetrominoKind::ALL {
            for rot in [Rotation::R0, Rotation::R1, Rotation::R2, Rotation::R3] {
                let c = cells(kind, rot);
                assert_eq!(c.len(), 4);
            }
        }
    }

    #[test]
    fn rotation_round_trip() {
        let r = Rotation::R0;
        assert_eq!(r.cw().cw().cw().cw(), Rotation::R0);
        assert_eq!(r.ccw().ccw().ccw().ccw(), Rotation::R0);
        assert_eq!(r.cw().ccw(), Rotation::R0);
    }

    #[test]
    fn o_piece_same_all_rotations() {
        let base = cells(TetrominoKind::O, Rotation::R0);
        for rot in [Rotation::R1, Rotation::R2, Rotation::R3] {
            assert_eq!(cells(TetrominoKind::O, rot), base);
        }
    }
}
