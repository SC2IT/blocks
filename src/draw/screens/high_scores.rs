use ggez::glam::Vec2;
use ggez::graphics::{Canvas, Color, Rect};
use ggez::GameResult;

use crate::scores::{ScoreBoard, MAX_SCORES};

use super::*;
use crate::draw::*;

pub(super) const SCORES_PANEL_WIDTH: f32 = 680.0;
pub(super) const SCORES_ROW_HEIGHT: f32 = 40.0;
pub(super) const SCORES_TEXT_SIZE: f32 = 24.0;
// right edges of the rank, initials, score, level and lines columns
pub(super) const SCORES_COLUMNS: [f32; 5] = [90.0, 210.0, 430.0, 530.0, 630.0];
pub(super) const EMPTY_ROW_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.35);

/// top scores, with the new entry highlighted
/// high score table
pub struct HighScoresScreen<'a> {
    pub scores: &'a ScoreBoard,
    pub highlight: Option<usize>,
}

impl Layer for HighScoresScreen<'_> {
    fn draw(&self, canvas: &mut Canvas, frame: &Frame) -> GameResult {
        let Self { scores, highlight } = *self;
        let ctx = frame.ctx;
        let meshes = frame.meshes;
        let view_rect = &frame.view.view_rect;
        let style = frame.block;
        let panel = menu_panel_rect(
            view_rect,
            MAX_SCORES + 1,
            view_rect.h * 0.2,
            SCORES_PANEL_WIDTH,
            SCORES_ROW_HEIGHT,
        );
        draw_title(
            ctx,
            canvas,
            "High Scores",
            panel.y - PAUSED_TITLE_SIZE * 0.75,
            PAUSED_TITLE_SIZE,
            view_rect,
        )?;
        draw_panel(canvas, meshes, panel);

        // right aligned cells
        let draw_row = |canvas: &mut Canvas,
                        row: Rect,
                        cells: [&str; 5],
                        size: f32,
                        color: Color|
         -> GameResult {
            for (cell, right) in cells.iter().zip(SCORES_COLUMNS) {
                let text = ui_text(cell, size);
                let dims = text.measure_ui(ctx)?;
                let pos = Vec2::new(panel.x + right - dims.x, row.y + (row.h - dims.y) / 2.0);
                draw_shadowed(canvas, &text, pos, color);
            }
            Ok(())
        };
        let header = menu_row_rect(panel, 0, SCORES_ROW_HEIGHT);
        draw_row(
            canvas,
            header,
            ["#", "Name", "Score", "Level", "Lines"],
            SMALL_TEXT_SIZE,
            LABEL_COLOR,
        )?;

        let pulse = 0.5 + 0.5 * (ctx.time.time_since_start().as_secs_f32() * 5.0).sin();
        for i in 0..MAX_SCORES {
            let row = menu_row_rect(panel, i + 1, SCORES_ROW_HEIGHT);
            let rank = format!("{}.", i + 1);
            let Some(entry) = scores.scores.get(i) else {
                draw_row(
                    canvas,
                    row,
                    [&rank, "---", "---", "---", "---"],
                    SCORES_TEXT_SIZE,
                    EMPTY_ROW_COLOR,
                )?;
                continue;
            };
            let color = if highlight == Some(i) {
                draw_selection(ctx, canvas, meshes, row, style);
                mix_color(ACCENT_COLOR, Color::WHITE, pulse)
            } else {
                Color::WHITE
            };
            draw_row(
                canvas,
                row,
                [
                    &rank,
                    &entry.initials,
                    &entry.score.to_string(),
                    &entry.level.to_string(),
                    &entry.lines.to_string(),
                ],
                SCORES_TEXT_SIZE,
                color,
            )?;
        }
        draw_hint(ctx, canvas, "Press any key", view_rect)
    }
}
