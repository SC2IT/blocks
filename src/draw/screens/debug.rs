use ggez::graphics::{Color, Rect};
use ggez::GameResult;

use crate::draw::*;

pub(super) const DEBUG_ROW_HEIGHT: f32 = 26.0;
pub(super) const DEBUG_PANEL_GAP: f32 = 24.0; // below the hold and next previews
pub(super) const DEBUG_HINTS: [&str; 12] = [
    "Up/Down: pick",
    "Enter: restart",
    "R: replay on/off",
    "Left/Right: speed",
    "-/+: level",
    "Shift -/+: level x10",
    "P: pause",
    "B/S: styles",
    "H: hide panels",
    "F/Shift F: fps",
    "C: record frames",
    "Esc: back",
];

/// averaged cost of drawing a frame
#[derive(Debug, Clone, Copy)]
pub struct FrameStats {
    pub draws: usize,
    pub scene_ms: f32,    // building the draw list
    pub encode_ms: f32,   // ggez turning it into gpu commands
    pub gpu_wait_ms: f32, // blocked on the last frame's gpu work, near 0 when the gpu keeps up
}

/// animation viewer list and controls
/// animation viewer controls
pub struct DebugScreen<'a> {
    pub names: &'a [&'a str],
    pub selected: usize,
    pub speed: f64,
    pub paused: bool,
    pub replay: bool,
    pub level: usize,
    pub fps: u32,
    pub stats: Option<FrameStats>, // None until the first window is averaged
    pub recording: bool,
}

impl Layer for DebugScreen<'_> {
    fn draw(&self, canvas: &mut Painter, frame: &Frame) -> GameResult {
        let Self {
            names,
            selected,
            speed,
            paused,
            replay,
            level,
            fps,
            stats,
            recording,
        } = *self;
        let ctx = frame.ctx;
        let meshes = frame.meshes;
        let view_settings = frame.view;
        let style = frame.block;
        let view = &view_settings.view_rect;
        let top = view_settings.preview_rect.bottom() + DEBUG_PANEL_GAP;

        // animations, right of the playfield
        let left = view_settings.preview_rect.x;
        let width = (view.w - left - PANEL_PADDING).max(view_settings.preview_rect.w);
        let rows = names.len() + 4;
        let panel = Rect::new(
            left,
            top,
            width,
            rows as f32 * DEBUG_ROW_HEIGHT + 2.0 * PANEL_PADDING,
        );
        draw_panel(canvas, meshes, panel);
        let title = ui_text("Animations", SMALL_TEXT_SIZE);
        let title_row = menu_row_rect(panel, 0, DEBUG_ROW_HEIGHT);
        draw_text_centered(ctx, canvas, &title, title_row.center().into(), LABEL_COLOR)?;
        for (i, name) in names.iter().enumerate() {
            let row = menu_row_rect(panel, i + 1, DEBUG_ROW_HEIGHT);
            let chosen = i == selected;
            if chosen {
                draw_selection(ctx, canvas, meshes, row, style);
            }
            let color = if chosen { ACCENT_COLOR } else { Color::WHITE };
            let text = ui_text(name, SMALL_TEXT_SIZE);
            draw_text_centered(ctx, canvas, &text, row.center().into(), color)?;
        }
        let status = if paused {
            "Paused".to_string()
        } else {
            format!("Speed {speed}x")
        };
        let replay = if replay { "Replay On" } else { "Replay Off" };
        let level = format!("Level {level}  {fps} FPS");
        for (i, line) in [status.as_str(), replay, level.as_str()]
            .into_iter()
            .enumerate()
        {
            let row = menu_row_rect(panel, rows - 3 + i, DEBUG_ROW_HEIGHT);
            let text = ui_text(line, SMALL_TEXT_SIZE);
            draw_text_centered(ctx, canvas, &text, row.center().into(), LABEL_COLOR)?;
        }

        // controls, left of the playfield
        let right = view_settings.hold_rect.right();
        let width = (right - PANEL_PADDING).max(view_settings.hold_rect.w);
        let panel = Rect::new(
            right - width,
            top,
            width,
            DEBUG_HINTS.len() as f32 * DEBUG_ROW_HEIGHT + 2.0 * PANEL_PADDING,
        );
        draw_panel(canvas, meshes, panel);
        for (i, hint) in DEBUG_HINTS.iter().enumerate() {
            let row = menu_row_rect(panel, i, DEBUG_ROW_HEIGHT);
            let text = ui_text(hint, SMALL_TEXT_SIZE * 0.75);
            draw_text_centered(ctx, canvas, &text, row.center().into(), LABEL_COLOR)?;
        }

        // frame cost, under the controls
        let Some(stats) = stats else {
            return Ok(());
        };
        let lines = [
            if recording {
                format!("{} draws  REC", stats.draws)
            } else {
                format!("{} draws", stats.draws)
            },
            format!("CPU {:.2}ms", stats.scene_ms + stats.encode_ms),
            format!("scene {:.2} enc {:.2}", stats.scene_ms, stats.encode_ms),
            format!("GPU wait {:.2}ms", stats.gpu_wait_ms),
        ];
        let panel = Rect::new(
            panel.x,
            panel.bottom() + DEBUG_PANEL_GAP,
            panel.w,
            lines.len() as f32 * DEBUG_ROW_HEIGHT + 2.0 * PANEL_PADDING,
        );
        draw_panel(canvas, meshes, panel);
        for (i, line) in lines.iter().enumerate() {
            let row = menu_row_rect(panel, i, DEBUG_ROW_HEIGHT);
            let text = ui_text(line, SMALL_TEXT_SIZE * 0.75);
            draw_text_centered(ctx, canvas, &text, row.center().into(), LABEL_COLOR)?;
        }
        Ok(())
    }
}
