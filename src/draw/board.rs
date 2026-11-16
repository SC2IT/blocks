use ggez::glam::{IVec2, Vec2};
use ggez::graphics::{self, Canvas, Color, Rect};
use ggez::{Context, GameResult};

use crate::effects::{Effects, GameOver};
use crate::level_progress::LevelProgress;
use crate::playfield::{self, LineClear, Playfield, SlotState};
use crate::rustomino::{Rustomino, RustominoType};
use crate::score_display::ScoreDisplay;
use crate::style::{BlockStyle, BoardStyle, Style};
use crate::util;

use super::*;

pub(super) const PLAYFIELD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
pub(super) const PREVIEW_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
pub(super) const HOLD_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.5);
pub(super) const GRID_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.06);
pub(super) const DOT_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.18);
pub(super) const DOT_SIZE: f32 = 3.0;
pub(super) const CHECKER_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.035);
pub(super) const GHOST_FILL_ALPHA: f32 = 0.12;
pub(super) const GHOST_OUTLINE_ALPHA: f32 = 0.6;
pub(super) const SCORE_POP_SCALE: f32 = 0.3; // how much the score grows when it increases
pub(super) const SCORE_POPUP_OFFSET: f32 = 34.0; // where "+points" starts above the score
pub(super) const SCORE_POPUP_RISE: f32 = 40.0; // how far "+points" floats up
pub(super) const QUAD_GLINT_LEAD: f32 = 0.06; // glint peak after a block starts fading, fraction of the clear
pub(super) const QUAD_GLINT_WIDTH: f32 = 0.12; // fraction of the clear
pub(super) const QUAD_GLINT_ROW: f32 = 0.025; // delay per row up
pub(super) const CALLOUT_SIZE: f32 = 32.0;
pub(super) const CALLOUT_QUAD_SIZE: f32 = 48.0;
pub(super) const CALLOUT_POP_SCALE: f32 = 0.5;
pub(super) const TRAIL_SEGMENTS: usize = 8;
pub(super) const TRAIL_ALPHA: f32 = 0.45;
pub(super) const IMPACT_ALPHA: f32 = 0.6;
pub(super) const CURTAIN_FLASH: f32 = 0.6;
pub(super) const PREVIEW_ROW: i32 = playfield::PLAYFIELD_SLOTS[1] as i32; // curtain reaches hold and next last
pub(super) const BEST_OFFSET: f32 = 100.0; // above the score, clear of the popups

/// piece color, grayed and flashed by the game over curtain at row y
pub(super) fn block_color(rtype: RustominoType, game_over: Option<&GameOver>, y: i32) -> Color {
    let color = rtype.color();
    let Some(game_over) = game_over else {
        return color;
    };
    let gray = mix_color(color, util::rgb_to_grayscale(color), game_over.gray(y));
    mix_color(gray, Color::WHITE, CURTAIN_FLASH * game_over.flash(y))
}

/// fade out in a wipe from where the rustomino landed to the edges
pub(super) fn line_clear_alpha(line_clear: &LineClear, x: usize) -> f32 {
    let start = line_clear.wipe_start(x);
    (1.0 - (line_clear.progress() - start) / playfield::LINE_CLEAR_FADE).clamp(0.0, 1.0) as f32
}

pub(super) fn draw_playing_backgound(
    canvas: &mut Canvas,
    meshes: &Meshes,
    view_settings: &ViewSettings,
    board: BoardStyle,
    hold: bool,
) {
    // frames around the staging area and playfield, and the side panels
    let staging = view_settings.staging_rect;
    let playfield = view_settings.playfield_rect;
    let play_area = Rect::new(
        staging.x,
        staging.y,
        staging.w,
        playfield.y + playfield.h - staging.y,
    );
    let hold_rect = hold.then_some(view_settings.hold_rect);
    for rect in [Some(play_area), Some(view_settings.preview_rect), hold_rect]
        .into_iter()
        .flatten()
    {
        draw_rect(canvas, meshes, frame_rect(rect), FRAME_COLOR);
    }

    let backgrounds = [
        (view_settings.staging_rect, STAGING_BACKGROUND_COLOR),
        (view_settings.playfield_rect, PLAYFIELD_BACKGROUND_COLOR),
        (view_settings.preview_rect, PREVIEW_BACKGROUND_COLOR),
        (view_settings.hold_rect, HOLD_BACKGROUND_COLOR),
    ];
    for (rect, color) in backgrounds
        .into_iter()
        .filter(|(rect, _)| hold || *rect != view_settings.hold_rect)
    {
        draw_rect(canvas, meshes, rect, color);
    }

    let cell = BLOCK_SIZE + BLOCK_PADDING;
    let (cols, rows) = (
        playfield::PLAYFIELD_SLOTS[0],
        playfield::PLAYFIELD_SIZE[1] as usize,
    );
    match board {
        BoardStyle::Grid => {
            // lines in the gaps between cells
            for col in 1..cols {
                let x = playfield.x + col as f32 * cell;
                let (x, w) = snap_line(x);
                let line = Rect::new(x, playfield.y, w, playfield.h);
                draw_rect(canvas, meshes, line, GRID_COLOR);
            }
            for row in 1..rows {
                let y = playfield.y + playfield.h - 2.0 * BLOCK_PADDING - row as f32 * cell;
                let (y, h) = snap_line(y);
                let line = Rect::new(playfield.x, y, playfield.w, h);
                draw_rect(canvas, meshes, line, GRID_COLOR);
            }
        }
        BoardStyle::Dots => {
            // dots where the gaps cross
            for row in 1..rows {
                let y = playfield.y + playfield.h - 2.0 * BLOCK_PADDING - row as f32 * cell;
                for col in 1..cols {
                    let x = playfield.x + col as f32 * cell;
                    let dot = snap_rect(Rect::new(
                        x + (BLOCK_PADDING - DOT_SIZE) / 2.0,
                        y + (BLOCK_PADDING - DOT_SIZE) / 2.0,
                        DOT_SIZE,
                        DOT_SIZE,
                    ));
                    draw_rect(canvas, meshes, dot, DOT_COLOR);
                }
            }
        }
        BoardStyle::Checker => {
            for row in 0..rows {
                for col in (row % 2..cols).step_by(2) {
                    let rect = playfield_block_rect(
                        [col as i32, row as i32],
                        &view_settings.staging_rect,
                        &playfield,
                    );
                    draw_rect(canvas, meshes, rect, CHECKER_COLOR);
                }
            }
        }
        BoardStyle::Clean => {}
    }
}

pub(super) fn draw_playfield(
    canvas: &mut Canvas,
    meshes: &Meshes,
    playfield: &Playfield,
    view_settings: &ViewSettings,
    game_over: Option<&GameOver>,
    style: BlockStyle,
    ghost: bool,
) {
    let (staging_rect, playfield_rect) =
        (&view_settings.staging_rect, &view_settings.playfield_rect);
    // draw the locked blocks
    for (y, slots_x) in playfield.slots.iter().enumerate() {
        let clearing = playfield
            .line_clear
            .as_ref()
            .filter(|line_clear| line_clear.lines.contains(&y));
        for (x, slot) in slots_x.iter().enumerate() {
            if let SlotState::Locked(rtype) = slot {
                let mut color = block_color(*rtype, game_over, y as i32);
                if let Some(line_clear) = clearing {
                    let progress = line_clear.progress() as f32;
                    let flash = (1.0 - progress * 4.0).max(0.0);
                    let base = if line_clear.lines.len() >= 4 {
                        // glint riding the wipe outward
                        let bottom = line_clear.lines.iter().min().copied().unwrap_or(y);
                        let peak = line_clear.wipe_start(x) as f32
                            + QUAD_GLINT_LEAD
                            + (y - bottom) as f32 * QUAD_GLINT_ROW;
                        let glint = (1.0 - (progress - peak).abs() / QUAD_GLINT_WIDTH).max(0.0);
                        mix_color(LINE_CLEAR_COLOR, Color::WHITE, glint * 0.8)
                    } else {
                        LINE_CLEAR_COLOR
                    };
                    color = mix_color(base, Color::WHITE, flash);
                    color.a *= line_clear_alpha(line_clear, x);
                }
                let rect = playfield_block_rect([x as i32, y as i32], staging_rect, playfield_rect);
                draw_block(canvas, meshes, rect, color, style);
            }
        }
    }

    // draw the active rustomino over the locked blocks
    if let Some(active) = &playfield.active_rustomino {
        for block in active.playfield_slots() {
            let rect = playfield_block_rect([block[0], block[1]], staging_rect, playfield_rect);
            let color = block_color(active.rtype, game_over, block[1]);
            draw_block(canvas, meshes, rect, color, style);
        }
    }

    if let Some(ghost) = playfield.ghost_rustomino.as_ref().filter(|_| ghost) {
        let color = block_color(ghost.rtype, game_over, PREVIEW_ROW);
        let slots = ghost.playfield_slots();
        if style == BlockStyle::Jelly {
            // rounded cells like the blocks
            for block in &slots {
                let rect = playfield_block_rect([block[0], block[1]], staging_rect, playfield_rect);
                draw_rounded(canvas, meshes, rect, with_alpha(color, GHOST_FILL_ALPHA));
                let param = graphics::DrawParam::default()
                    .dest(Vec2::new(rect.x, rect.y))
                    .scale(Vec2::new(rect.w / BLOCK_SIZE, rect.h / BLOCK_SIZE))
                    .color(with_alpha(color, GHOST_OUTLINE_ALPHA));
                draw_mesh(canvas, &meshes.rounded_outline, param);
            }
            return;
        }
        for block in &slots {
            let rect = playfield_block_rect([block[0], block[1]], staging_rect, playfield_rect);
            draw_rect(canvas, meshes, rect, with_alpha(color, GHOST_FILL_ALPHA));
        }
        let outline = with_alpha(color, GHOST_OUTLINE_ALPHA);
        for rect in ghost_outline(&slots, staging_rect, playfield_rect) {
            draw_rect(canvas, meshes, rect, outline);
        }
    }
}

/// fixed width outline inside the ghost piece's boundary, as non-overlapping rects
pub(super) fn ghost_outline(
    slots: &[IVec2],
    staging_rect: &Rect,
    playfield_rect: &Rect,
) -> Vec<Rect> {
    let s = block_size();
    let (p, w) = (CELL - s, snap(GHOST_OUTLINE_WIDTH));
    let has = |x: i32, y: i32| slots.contains(&IVec2::new(x, y));
    let mut rects = Vec::new();
    for &IVec2 { x, y } in slots {
        let r = playfield_block_rect([x, y], staging_rect, playfield_rect);
        // slot y+1 is above on screen
        let (l, rt, u, d) = (has(x - 1, y), has(x + 1, y), has(x, y + 1), has(x, y - 1));
        let (ul, ur, dl, dr) = (
            has(x - 1, y + 1),
            has(x + 1, y + 1),
            has(x - 1, y - 1),
            has(x + 1, y - 1),
        );

        // sides, verticals trimmed where they meet horizontals
        if !u {
            rects.push(Rect::new(r.x, r.y, s, w));
        }
        if !d {
            rects.push(Rect::new(r.x, r.y + s - w, s, w));
        }
        let top = if u { 0.0 } else { w };
        let side_h = s - top - if d { 0.0 } else { w };
        if !l {
            rects.push(Rect::new(r.x, r.y + top, w, side_h));
        }
        if !rt {
            rects.push(Rect::new(r.x + s - w, r.y + top, w, side_h));
        }

        // inner corners
        if l && u && !ul {
            rects.push(Rect::new(r.x, r.y, w, w));
        }
        if rt && u && !ur {
            rects.push(Rect::new(r.x + s - w, r.y, w, w));
        }
        if l && d && !dl {
            rects.push(Rect::new(r.x, r.y + s - w, w, w));
        }
        if rt && d && !dr {
            rects.push(Rect::new(r.x + s - w, r.y + s - w, w, w));
        }

        // bridge the padding to the right and above
        if rt {
            if !(u && ur) {
                rects.push(Rect::new(r.x + s, r.y, p, w));
            }
            if !(d && dr) {
                rects.push(Rect::new(r.x + s, r.y + s - w, p, w));
            }
        }
        if u {
            if !(l && ul) {
                rects.push(Rect::new(r.x, r.y - p, w, p));
            }
            if !(rt && ur) {
                rects.push(Rect::new(r.x + s - w, r.y - p, w, p));
            }
        }
    }
    rects
}

pub(super) fn draw_preview(
    canvas: &mut Canvas,
    meshes: &Meshes,
    rustomino: &Option<Rustomino>,
    preview_rect: &Rect,
    game_over: Option<&GameOver>,
    style: BlockStyle,
) {
    if let Some(rustomino) = rustomino {
        // center the piece's bounding box in the panel
        let cell = BLOCK_SIZE + BLOCK_PADDING;
        let min = rustomino
            .blocks
            .iter()
            .fold(rustomino.blocks[0], |a, b| a.min(*b));
        let max = rustomino
            .blocks
            .iter()
            .fold(rustomino.blocks[0], |a, b| a.max(*b));
        let width = (max.x - min.x + 1) as f32 * cell - BLOCK_PADDING;
        let height = (max.y - min.y + 1) as f32 * cell - BLOCK_PADDING;
        let left = snap(preview_rect.x + (preview_rect.w - width) / 2.0);
        let top = snap(preview_rect.y + (preview_rect.h - height) / 2.0);
        for block in rustomino.blocks {
            let rect = Rect::new(
                left + (block.x - min.x) as f32 * cell,
                top + (max.y - block.y) as f32 * cell,
                block_size(),
                block_size(),
            );
            draw_block(
                canvas,
                meshes,
                rect,
                block_color(rustomino.rtype, game_over, PREVIEW_ROW),
                style,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_playing(
    canvas: &mut Canvas,
    meshes: &Meshes,
    playfield: &Playfield,
    next_rustomino: &Option<Rustomino>,
    hold_rustomino: &Option<Rustomino>,
    view_settings: &ViewSettings,
    game_over: Option<&GameOver>,
    style: &Style,
    assists: bool, // ghost and hold
) {
    draw_playing_backgound(canvas, meshes, view_settings, style.board, assists);
    draw_playfield(
        canvas,
        meshes,
        playfield,
        view_settings,
        game_over,
        style.block,
        assists,
    );
    if assists {
        draw_preview(
            canvas,
            meshes,
            hold_rustomino,
            &view_settings.hold_rect,
            game_over,
            style.block,
        );
    }
    draw_preview(
        canvas,
        meshes,
        next_rustomino,
        &view_settings.preview_rect,
        game_over,
        style.block,
    );
}

pub(super) fn draw_playing_text(
    canvas: &mut Canvas,
    level: &ScoreDisplay,
    score: &ScoreDisplay,
    best: usize,
    view_settings: &ViewSettings,
    hold_shown: bool,
) -> GameResult {
    let label_size = UI_FONT_SIZE * 0.75;
    let label_offset = Vec2::new(0.0, -(label_size + FRAME_WIDTH + 8.0));
    let hold = view_settings.hold_rect;
    let preview = view_settings.preview_rect;

    draw_piece_colored(canvas, "Blocks!", view_settings.title_pos, UI_FONT_SIZE);
    if hold_shown {
        draw_text(
            canvas,
            "Hold",
            Vec2::new(hold.x, hold.y) + label_offset,
            label_size,
            LABEL_COLOR,
        );
    }
    draw_text(
        canvas,
        "Next",
        Vec2::new(preview.x, preview.y) + label_offset,
        label_size,
        LABEL_COLOR,
    );
    draw_text(
        canvas,
        "Level:",
        view_settings.level_label_pos,
        UI_FONT_SIZE,
        LABEL_COLOR,
    );
    draw_text(
        canvas,
        "Score:",
        view_settings.score_label_pos,
        UI_FONT_SIZE,
        LABEL_COLOR,
    );
    draw_counter(canvas, level, view_settings.level_pos);
    draw_counter(canvas, score, view_settings.score_pos);
    let best_offset = Vec2::new(0.0, -BEST_OFFSET);
    draw_text(
        canvas,
        "Best:",
        view_settings.score_label_pos + best_offset,
        label_size,
        LABEL_COLOR,
    );
    draw_text(
        canvas,
        &best.to_string(),
        view_settings.score_pos + best_offset,
        label_size,
        Color::WHITE,
    );

    Ok(())
}

/// value that counts up and pops when it increases, with rising "+amount"
pub(super) fn draw_counter(canvas: &mut Canvas, score: &ScoreDisplay, pos: Vec2) {
    let pop = score.pop();
    let size = UI_FONT_SIZE * (1.0 + SCORE_POP_SCALE * pop);
    let grow = Vec2::new(0.0, (size - UI_FONT_SIZE) / 2.0);
    let color = mix_color(Color::WHITE, ACCENT_COLOR, pop);
    draw_text(canvas, &score.shown().to_string(), pos - grow, size, color);

    for popup in &score.popups {
        let t = popup.progress();
        let rise = Vec2::new(0.0, SCORE_POPUP_OFFSET + SCORE_POPUP_RISE * t);
        let color = with_alpha(ACCENT_COLOR, 1.0 - t * t);
        let text = format!("+{}", popup.points);
        draw_text(canvas, &text, pos - rise, UI_FONT_SIZE * 0.8, color);
    }
}

/// segments under the playfield filled per line toward the next level
pub(super) fn draw_level_progress(
    canvas: &mut Canvas,
    meshes: &Meshes,
    progress: &LevelProgress,
    view_settings: &ViewSettings,
    style: BlockStyle,
) {
    let bar = view_settings.progress_rect;
    let burst = progress.burst();
    let frame_color = mix_color(FRAME_COLOR, ACCENT_COLOR, burst);
    draw_rect(canvas, meshes, frame_rect(bar), frame_color);
    draw_rect(canvas, meshes, bar, PLAYFIELD_BACKGROUND_COLOR);

    // whole pixel segments and gaps, centered in the bar
    let per_level = progress.len();
    let gap = snap(BLOCK_PADDING).max(1.0 / pixel_scale());
    let pitch = snap((bar.w - gap) / per_level as f32);
    let left = bar.x + snap((bar.w - gap - pitch * per_level as f32) / 2.0) + gap;
    let (top, height) = (snap(bar.y + BLOCK_PADDING), snap(PROGRESS_HEIGHT));
    let center_y = top + height / 2.0;
    let pulse = progress.pulse();
    for i in 0..per_level {
        let rect = Rect::new(left + i as f32 * pitch, top, pitch - gap, height);
        draw_rect(canvas, meshes, rect, SLIDER_EMPTY_COLOR);
        if let Some((_, amount)) = pulse.filter(|(at, _)| *at == i) {
            draw_rect(
                canvas,
                meshes,
                rect,
                with_alpha(ACCENT_COLOR, amount * 0.35),
            );
        }
        let Some(look) = progress.segment(i) else {
            continue;
        };
        let (w, h) = (rect.w * look.width, rect.h * look.height);
        let sized = snap_rect(Rect::new(
            rect.center().x - w / 2.0,
            center_y - h / 2.0,
            w,
            h,
        ));
        // soft two layer halo
        for (spread, alpha) in [(4.0, 0.12), (2.0, 0.22)] {
            if look.glow <= 0.0 {
                break;
            }
            let spread = spread * (0.5 + look.glow);
            let halo = Rect::new(
                sized.x - spread,
                sized.y - spread,
                sized.w + 2.0 * spread,
                sized.h + 2.0 * spread,
            );
            draw_rect(
                canvas,
                meshes,
                snap_rect(halo),
                with_alpha(ACCENT_COLOR, look.glow * alpha),
            );
        }
        let color = mix_color(ACCENT_COLOR, Color::WHITE, look.heat);
        draw_block(canvas, meshes, sized, color, style);
    }

    for spark in &progress.sparks {
        let size = snap(spark.size);
        let x = left + spark.pos.x * pitch - gap / 2.0 - size / 2.0;
        let y = center_y + spark.pos.y - size / 2.0;
        let color = if spark.white {
            Color::WHITE
        } else {
            ACCENT_COLOR
        };
        draw_rect(
            canvas,
            meshes,
            snap_rect(Rect::new(x, y, size, size)),
            with_alpha(color, spark.fade()),
        );
    }
}

/// particles, playfield flash and line clear callout
pub(super) fn draw_effects(
    ctx: &Context,
    canvas: &mut Canvas,
    meshes: &Meshes,
    effects: &Effects,
    view_settings: &ViewSettings,
) -> GameResult {
    let playfield = view_settings.playfield_rect;
    let flash = effects.flash();
    if flash > 0.0 {
        draw_rect(canvas, meshes, playfield, with_alpha(Color::WHITE, flash));
    }
    let cell = BLOCK_SIZE + BLOCK_PADDING;
    let left = view_settings.staging_rect.x + 1.0;
    let bottom = playfield.y + playfield.h - 1.0;
    for trail in &effects.trails {
        let fade = trail.fade();
        let x = left + trail.x * cell;
        let length = trail.from - trail.to;
        for i in 0..TRAIL_SEGMENTS {
            let t = i as f32 / TRAIL_SEGMENTS as f32;
            let y0 = trail.to + length * t;
            let h = length / TRAIL_SEGMENTS as f32 * cell;
            let rect = snap_edges(Rect::new(x, bottom - y0 * cell - h, cell, h));
            let alpha = TRAIL_ALPHA * trail.strength * fade * (1.0 - t);
            draw_rect(canvas, meshes, rect, with_alpha(trail.color, alpha));
        }
    }
    if let Some(impact) = &effects.impact {
        let color = with_alpha(Color::WHITE, IMPACT_ALPHA * impact.strength * impact.fade());
        for block in impact.blocks {
            let rect =
                playfield_block_rect([block.x, block.y], &view_settings.staging_rect, &playfield);
            draw_rect(canvas, meshes, rect, color);
        }
    }
    for particle in effects.particles.iter().filter(|p| p.visible()) {
        let fade = particle.fade();
        let size = BLOCK_SIZE * particle.size * (0.4 + 0.6 * fade);
        let center = Vec2::new(left + particle.pos.x * cell, bottom - particle.pos.y * cell);
        let rect = Rect::new(center.x - size / 2.0, center.y - size / 2.0, size, size);
        draw_rect(
            canvas,
            meshes,
            snap_rect(rect),
            with_alpha(particle.color, fade),
        );
    }

    if let Some(callout) = &effects.callout {
        let t = callout.progress();
        let quad = callout.lines >= 4;
        let base = if quad {
            CALLOUT_QUAD_SIZE
        } else {
            CALLOUT_SIZE
        };
        let mut size = base * (1.0 + CALLOUT_POP_SCALE * callout.pop());
        let width = ui_text(callout.text(), size).measure_ui(ctx)?.x;
        size *= ((playfield.w - 2.0 * PANEL_PADDING) / width).min(1.0);
        let alpha = ((1.0 - t) / 0.4).min(1.0);
        let center = Vec2::new(
            playfield.x + playfield.w / 2.0,
            bottom - (callout.row + t) * cell,
        );
        if quad {
            let (text, shadow) = piece_colored_text_alpha(callout.text(), size, alpha);
            let dims = text.measure_ui(ctx)?;
            let pos = center - Vec2::new(dims.x / 2.0, dims.y / 2.0);
            let param = text_param();
            draw(canvas, &shadow, param.dest(pos + Vec2::splat(3.0)));
            draw(canvas, &text, param.dest(pos));
        } else {
            let color = with_alpha(LINE_CLEAR_COLOR, alpha);
            draw_text_centered(ctx, canvas, &ui_text(callout.text(), size), center, color)?;
        }
    }
    Ok(())
}

pub(super) fn playfield_block_rect(
    block: [i32; 2],
    staging_rect: &Rect,
    playfield_rect: &Rect,
) -> Rect {
    // block[x,y] absolute units
    let x = staging_rect.x + (block[0] as f32 * (BLOCK_SIZE + BLOCK_PADDING)) + 1.0;
    // get bottom left of playfield_rect
    let y = playfield_rect.y + playfield_rect.h
        - ((block[1] + 1) as f32 * (BLOCK_SIZE + BLOCK_PADDING))
        - 1.0;

    Rect::new(snap(x), snap(y), block_size(), block_size())
}

/// the board with its previews, progress, effects and text
pub struct GameLayer<'a> {
    pub playfield: &'a Playfield,
    pub next: &'a Option<Rustomino>,
    pub held: &'a Option<Rustomino>,
    pub game_over: Option<&'a GameOver>,
    pub style: &'a Style,
    pub assists: bool, // ghost and hold
    pub level_progress: &'a LevelProgress,
    pub effects: &'a Effects,
    pub level_display: &'a ScoreDisplay,
    pub score_display: &'a ScoreDisplay,
    pub best: usize,
    pub shake: Option<Vec2>, // None when not shaking
}

impl Layer for GameLayer<'_> {
    fn draw(&self, canvas: &mut Canvas, frame: &Frame) -> GameResult {
        let screen = canvas.screen_coordinates();
        if let (Some(rect), Some(shake)) = (screen, self.shake) {
            set_screen_coordinates(
                canvas,
                Rect::new(rect.x - shake.x, rect.y - shake.y, rect.w, rect.h),
            );
        }
        draw_playing(
            canvas,
            frame.meshes,
            self.playfield,
            self.next,
            self.held,
            frame.view,
            self.game_over,
            self.style,
            self.assists,
        );
        draw_level_progress(
            canvas,
            frame.meshes,
            self.level_progress,
            frame.view,
            self.style.block,
        );
        draw_effects(frame.ctx, canvas, frame.meshes, self.effects, frame.view)?;
        draw_playing_text(
            canvas,
            self.level_display,
            self.score_display,
            self.best,
            frame.view,
            self.assists,
        )?;
        if let Some(rect) = screen {
            set_screen_coordinates(canvas, rect);
        }
        Ok(())
    }
}
