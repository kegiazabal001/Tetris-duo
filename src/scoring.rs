use bevy::prelude::*;

use crate::config::AppConfig;
use crate::piece::TSpinType;
use crate::player::LinesCleared;
use crate::state::SelectedMode;

#[derive(Event)]
pub struct LevelUpEvent {
    pub new_level: u32,
}

#[derive(Resource, Debug)]
pub struct ScoreBoard {
    pub score: u32,
    pub lines_cleared: u32,
    pub level: u32,
    /// Active combo counter. -1 = no combo (resets when a piece locks with 0 lines cleared).
    pub combo: i32,
    /// True if the last "hard" clear was a Tetris (4 lines) or Full T-Spin.
    /// Used to award Back-to-Back bonus.
    pub last_was_hard_clear: bool,
    pub high_score: u32,
}

impl Default for ScoreBoard {
    fn default() -> Self {
        Self {
            score: 0,
            lines_cleared: 0,
            level: 1,
            combo: -1,
            last_was_hard_clear: false,
            high_score: 0,
        }
    }
}

pub fn save_high_score(
    mut score: ResMut<ScoreBoard>,
    mut config: ResMut<AppConfig>,
    mode: Res<SelectedMode>,
) {
    if *mode == SelectedMode::Endless && config.try_update_endless(score.score) {
        score.high_score = score.score;
    }
}

impl ScoreBoard {
    /// Non-linear level progression: early levels require fewer lines.
    /// Thresholds are cumulative lines needed to *reach* each level.
    pub fn level_from_lines(lines: u32) -> u32 {
        // Lines to reach levels 1–5: 0, 5, 12, 20, 29
        const THRESHOLDS: &[u32] = &[0, 5, 12, 20, 29];
        if lines >= 29 {
            return 5 + (lines - 29) / 10;
        }
        for (i, &threshold) in THRESHOLDS.iter().enumerate().rev() {
            if lines >= threshold {
                return (i as u32 + 1).max(1);
            }
        }
        1
    }

    /// Resets the scoreboard to defaults while preserving the all-time high score.
    pub fn reset_preserving_high_score(&mut self) {
        let high_score = self.high_score;
        *self = ScoreBoard::default();
        self.high_score = high_score;
    }

    /// NES-style gravity interval (seconds per row drop).
    pub fn gravity_interval(&self) -> f32 {
        match self.level {
            0 | 1 => 0.60,
            2 => 0.48,
            3 => 0.38,
            4 => 0.30,
            5 => 0.23,
            6 => 0.18,
            7 => 0.14,
            8 => 0.10,
            9 => 0.08,
            10..=12 => 0.06,
            13..=15 => 0.05,
            16..=18 => 0.04,
            19..=28 => 0.03,
            _ => 0.02,
        }
    }
}

/// System: update score from LinesCleared events.
pub fn update_score(
    mut score: ResMut<ScoreBoard>,
    mut ev: EventReader<LinesCleared>,
    mut ev_level_up: EventWriter<LevelUpEvent>,
) {
    for event in ev.read() {
        if event.count == 0 {
            // Piece locked without clearing — reset combo
            score.combo = -1;
            continue;
        }

        // Base points (Tetris Guideline)
        let base = match (event.count, event.t_spin) {
            (_, TSpinType::Full) => match event.count {
                1 => 800,
                2 => 1200,
                3 => 1600,
                _ => 2400,
            },
            (_, TSpinType::Mini) => match event.count {
                1 => 200,
                2 => 400,
                _ => 100,
            },
            (1, _) => 100,
            (2, _) => 300,
            (3, _) => 500,
            (4, _) => 800, // Tetris
            _ => event.count * 200,
        };

        // Back-to-Back bonus (×1.5 for consecutive hard clears)
        let is_hard_clear =
            event.count == 4 || matches!(event.t_spin, TSpinType::Full | TSpinType::Mini);
        let btb_mult = if is_hard_clear && score.last_was_hard_clear {
            3 // ×1.5 implemented as ×3/2
        } else {
            2
        };
        score.last_was_hard_clear = is_hard_clear;

        // Combo bonus: 50 × combo × level
        score.combo += 1;
        let combo_bonus = if score.combo > 0 {
            50 * score.combo as u32 * score.level
        } else {
            0
        };

        score.score += base * score.level * btb_mult / 2 + combo_bonus;
        score.lines_cleared += event.count;
        let new_level = ScoreBoard::level_from_lines(score.lines_cleared);
        if new_level > score.level {
            score.level = new_level;
            ev_level_up.write(LevelUpEvent { new_level });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::TSpinType;
    use crate::player::LinesCleared;

    fn cleared(count: u32, t_spin: TSpinType) -> LinesCleared {
        LinesCleared { player: crate::player::PlayerId::P1, count, t_spin }
    }

    fn score_for(events: &[LinesCleared]) -> u32 {
        let mut sb = ScoreBoard::default();
        for ev in events {
            // Replicate update_score logic inline so we can test without Bevy App
            if ev.count == 0 {
                sb.combo = -1;
                continue;
            }
            let base = match (ev.count, ev.t_spin) {
                (_, TSpinType::Full) => match ev.count {
                    1 => 800,
                    2 => 1200,
                    3 => 1600,
                    _ => 2400,
                },
                (_, TSpinType::Mini) => match ev.count {
                    1 => 200,
                    2 => 400,
                    _ => 100,
                },
                (1, _) => 100,
                (2, _) => 300,
                (3, _) => 500,
                (4, _) => 800,
                _ => ev.count * 200,
            };
            let is_hard = ev.count == 4 || matches!(ev.t_spin, TSpinType::Full | TSpinType::Mini);
            let btb = if is_hard && sb.last_was_hard_clear { 3 } else { 2 };
            sb.last_was_hard_clear = is_hard;
            sb.combo += 1;
            let combo_bonus = if sb.combo > 0 { 50 * sb.combo as u32 * sb.level } else { 0 };
            sb.score += base * sb.level * btb / 2 + combo_bonus;
            sb.lines_cleared += ev.count;
            sb.level = ScoreBoard::level_from_lines(sb.lines_cleared);
        }
        sb.score
    }

    #[test]
    fn single_line_score() {
        // 1 line at level 1, no BtB, no combo → 100 × 1 = 100
        assert_eq!(score_for(&[cleared(1, TSpinType::None)]), 100);
    }

    #[test]
    fn tetris_score() {
        // 4 lines at level 1 → base 800, btb_mult=2/2=1 → 800
        assert_eq!(score_for(&[cleared(4, TSpinType::None)]), 800);
    }

    #[test]
    fn back_to_back_bonus() {
        // Tetris then Tetris → second gets ×1.5 (btb_mult 3/2) + combo bonus 50×1×1.
        // First:  800*1*2/2 + 0   = 800 (combo=-1→0, no bonus)
        // Second: 800*1*3/2 + 50  = 1200 + 50 = 1250
        let s = score_for(&[cleared(4, TSpinType::None), cleared(4, TSpinType::None)]);
        assert_eq!(s, 800 + 1250);
    }

    #[test]
    fn combo_counter_increments() {
        // Two consecutive 1-line clears: second has combo=1 → combo_bonus = 50×1×1 = 50
        let s = score_for(&[cleared(1, TSpinType::None), cleared(1, TSpinType::None)]);
        // First: 100, Second: 100 + 50 = 150 → total 250
        assert_eq!(s, 250);
    }

    #[test]
    fn combo_resets_on_no_clear() {
        // Clear, no-clear, clear — second clear should have no combo bonus
        let s = score_for(&[
            cleared(1, TSpinType::None),
            cleared(0, TSpinType::None), // resets combo
            cleared(1, TSpinType::None),
        ]);
        // 100 + 0 + 100 = 200 (no combo bonuses)
        assert_eq!(s, 200);
    }

    #[test]
    fn test_level_from_lines_early_curve() {
        assert_eq!(ScoreBoard::level_from_lines(0), 1);
        assert_eq!(ScoreBoard::level_from_lines(4), 1);
        assert_eq!(ScoreBoard::level_from_lines(5), 2);
        assert_eq!(ScoreBoard::level_from_lines(11), 2);
        assert_eq!(ScoreBoard::level_from_lines(12), 3);
        assert_eq!(ScoreBoard::level_from_lines(19), 3);
        assert_eq!(ScoreBoard::level_from_lines(20), 4);
        assert_eq!(ScoreBoard::level_from_lines(28), 4);
        assert_eq!(ScoreBoard::level_from_lines(29), 5);
    }

    #[test]
    fn test_level_from_lines_late_curve() {
        // Level 5+: every 10 lines above 29
        assert_eq!(ScoreBoard::level_from_lines(39), 6);
        assert_eq!(ScoreBoard::level_from_lines(49), 7);
        assert_eq!(ScoreBoard::level_from_lines(38), 5);
    }

    #[test]
    fn t_spin_full_beats_tetris() {
        // T-Spin Full 1 line = 800 base; Tetris = 800 base but T-Spin triggers BtB differently.
        // Key assertion: T-Spin Full base for 1 line (800) equals Tetris base (800), both "hard clears".
        // After a Tetris, T-Spin Full 1 line with BtB = 800 × 1.5 = 1200 > next plain line.
        let s_tspin = score_for(&[cleared(4, TSpinType::None), cleared(1, TSpinType::Full)]);
        let s_normal = score_for(&[cleared(4, TSpinType::None), cleared(1, TSpinType::None)]);
        assert!(s_tspin > s_normal, "T-Spin Full should score more than single line after Tetris");
    }

    #[test]
    fn t_spin_4_lines_score() {
        // T-Spin Full 4 lines at level 1, no BtB → base 2400, btb_mult=2/2=1 → 2400
        assert_eq!(score_for(&[cleared(4, TSpinType::Full)]), 2400);
    }

    #[test]
    fn t_spin_4_lines_greater_than_tetris() {
        // T-Spin Full 4 lines (2400) > plain Tetris (800) at same level
        assert!(
            score_for(&[cleared(4, TSpinType::Full)]) > score_for(&[cleared(4, TSpinType::None)]),
            "T-Spin Full 4 lines should score more than plain Tetris"
        );
    }
}
