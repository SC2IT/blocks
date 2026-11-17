use ggez::glam::{IVec2, Vec2};
use ggez::graphics::{self, Color, Rect};
use ggez::{Context, GameResult};

use crate::background::Background;
use crate::effects::{DropStyle, Effects, GameOver};
use crate::level_progress::LevelProgress;
use crate::playfield::{self, LineClear, Playfield, SlotState, Wear};
use crate::rustomino::{Rustomino, RustominoType};
use crate::score_display::ScoreDisplay;
use crate::style::{BlockStyle, BoardStyle, Style};
use crate::util;
use strum::IntoEnumIterator;

use super::tough::{crack_color, draw_tough, ToughScene};
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
// glitch pieces: split into magenta and cyan, with bursts of jumps, recolors and tears
pub(super) const GLITCH_MAGENTA: Color = Color::new(1.0, 0.2, 0.75, 1.0);
pub(super) const GLITCH_CYAN: Color = Color::new(0.2, 0.95, 1.0, 1.0);
pub(super) const GLITCH_SPLIT: f32 = 3.0; // px between the block and each color copy
pub(super) const GLITCH_SPLIT_ALPHA: f32 = 0.75;
pub(super) const GLITCH_RATE: f32 = 12.0; // glitch frames per second
pub(super) const GLITCH_BURST: f32 = 0.2; // share of frames a block bursts
pub(super) const GLITCH_JUMP: f32 = 0.2; // farthest a bursting block jumps, in blocks
pub(super) const GLITCH_TEAR: f32 = 0.12; // height of a tear, in blocks
pub(super) const GLITCH_LOCKED: f32 = 0.3; // locked glitch blocks burst this much as often
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
    canvas: &mut Painter,
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

/// marks for tough and cracked blocks, until they get their final look
/// marks for tough blocks under the game over curtain, and the crack of a cracked block in
/// a trace of the scene it showed
fn draw_wear(
    canvas: &mut Painter,
    meshes: &Meshes,
    rect: Rect,
    color: Color,
    wear: Wear,
    crack: Color,
) {
    match wear {
        Wear::Whole => {}
        Wear::Tough => {
            // a dark core with a light center, inside the block's own color
            let core = inset(rect, rect.w * 0.26);
            draw_rect(
                canvas,
                meshes,
                core,
                with_alpha(shade(color, 0.35), color.a),
            );
            let center = inset(rect, rect.w * 0.42);
            draw_rect(
                canvas,
                meshes,
                center,
                with_alpha(tint(color, 0.3), color.a),
            );
        }
        Wear::Cracked => {
            // a jagged crack from corner to corner
            let dark = with_alpha(crack, color.a);
            let size = (3.0 * rect.w / BLOCK_SIZE).max(1.0);
            let jags = [0.0, 0.08, -0.04, 0.1, 0.0, -0.08, 0.05, 0.0, 0.06];
            for (i, jag) in jags.iter().enumerate() {
                let t = i as f32 / (jags.len() - 1) as f32;
                let x = rect.x + rect.w * (0.18 + 0.64 * t) - size / 2.0;
                let y = rect.y + rect.h * (0.2 + 0.6 * t + jag) - size / 2.0;
                draw_rect(canvas, meshes, snap_rect(Rect::new(x, y, size, size)), dark);
            }
        }
    }
}

/// a tough piece's blocks for drawing, from (column, row, rect) with rows counting up: each
/// block grown over the gaps to the piece's other blocks, whether nothing of the piece is
/// below it, and the piece's bounds
fn piece_faces(blocks: &[(i32, i32, Rect)]) -> (Vec<(Rect, bool)>, Rect) {
    let find = |x: i32, y: i32| blocks.iter().find(|b| b.0 == x && b.1 == y).map(|b| b.2);
    let faces = blocks
        .iter()
        .map(|&(x, y, rect)| {
            let mut face = rect;
            if let Some(right) = find(x + 1, y) {
                face.w = right.x - rect.x;
            }
            if let Some(below) = find(x, y - 1) {
                face.h = below.y - rect.y;
            }
            (face, find(x, y - 1).is_none())
        })
        .collect();
    let left = blocks.iter().map(|b| b.2.x).fold(f32::MAX, f32::min);
    let top = blocks.iter().map(|b| b.2.y).fold(f32::MAX, f32::min);
    let right = blocks.iter().map(|b| b.2.right()).fold(f32::MIN, f32::max);
    let bottom = blocks.iter().map(|b| b.2.bottom()).fold(f32::MIN, f32::max);
    (faces, Rect::new(left, top, right - left, bottom - top))
}

/// the tough blocks locked on the board, grouped into pieces: touching tough blocks of a kind
fn tough_pieces(playfield: &Playfield) -> Vec<Vec<(usize, usize)>> {
    let (w, h) = (playfield::PLAYFIELD_SLOTS[0], playfield.slots.len());
    let mut seen = vec![false; w * h];
    let mut pieces = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let SlotState::Locked(rtype, Wear::Tough) = playfield.slots[y][x] else {
                continue;
            };
            if seen[y * w + x] {
                continue;
            }
            let mut piece = Vec::new();
            let mut todo = vec![(x, y)];
            while let Some((cx, cy)) = todo.pop() {
                if seen[cy * w + cx]
                    || playfield.slots[cy][cx] != SlotState::Locked(rtype, Wear::Tough)
                {
                    continue;
                }
                seen[cy * w + cx] = true;
                piece.push((cx, cy));
                if cx > 0 {
                    todo.push((cx - 1, cy));
                }
                if cx + 1 < w {
                    todo.push((cx + 1, cy));
                }
                if cy > 0 {
                    todo.push((cx, cy - 1));
                }
                if cy + 1 < h {
                    todo.push((cx, cy + 1));
                }
            }
            pieces.push(piece);
        }
    }
    pieces
}

/// how often glitch pieces burst, none under the game over curtain or in low sensory
fn glitch_calm(game_over: Option<&GameOver>, scene: &ToughScene) -> f32 {
    if game_over.is_some() || scene.background.low_sensory {
        0.0
    } else {
        1.0
    }
}

/// a block of a glitch piece, calm scales how often it bursts
#[allow(clippy::too_many_arguments)]
fn draw_glitch_block(
    canvas: &mut Painter,
    meshes: &Meshes,
    rect: Rect,
    color: Color,
    style: BlockStyle,
    time: f32,
    seed: usize,
    calm: f32,
) {
    let frame = (time * GLITCH_RATE).floor();
    let n = |k: f32| hash(seed, frame * 0.731 + k);
    let burst = n(1.0) < GLITCH_BURST * calm;
    let (mut rect, mut color) = (rect, color);
    if burst {
        // jump sideways and maybe flash another piece's color
        rect.x += (n(2.0) * 2.0 - 1.0) * GLITCH_JUMP * rect.w;
        rect = snap_rect(rect);
        if n(3.0) < 0.5 {
            let types: Vec<RustominoType> = RustominoType::iter().collect();
            let other = types[(n(4.0) * types.len() as f32) as usize % types.len()];
            color = with_alpha(other.color(), color.a);
        }
    }
    let split = snap(GLITCH_SPLIT * rect.w / BLOCK_SIZE * if burst { 2.0 } else { 1.0 });
    let shifted = |dx: f32| Rect::new(rect.x + dx, rect.y, rect.w, rect.h);
    let alpha = GLITCH_SPLIT_ALPHA * color.a;
    draw_rect(
        canvas,
        meshes,
        shifted(-split),
        with_alpha(GLITCH_MAGENTA, alpha),
    );
    draw_rect(
        canvas,
        meshes,
        shifted(split),
        with_alpha(GLITCH_CYAN, alpha),
    );
    draw_block(canvas, meshes, rect, color, style);
    if burst && n(5.0) < 0.5 {
        // a bright tear across the block
        let h = GLITCH_TEAR * rect.h;
        let tear = Rect::new(
            rect.x - 2.0 * split,
            rect.y + n(6.0) * (rect.h - h),
            rect.w + 4.0 * split,
            h,
        );
        draw_rect(
            canvas,
            meshes,
            snap_rect(tear),
            with_alpha(Color::WHITE, 0.8 * color.a),
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_playfield(
    canvas: &mut Painter,
    meshes: &Meshes,
    playfield: &Playfield,
    view_settings: &ViewSettings,
    game_over: Option<&GameOver>,
    style: BlockStyle,
    ghost: bool,
    time: f32,
    scene: &ToughScene,
) {
    // glitch pieces stop glitching under the game over curtain, low sensory holds them split
    let calm = glitch_calm(game_over, scene);
    let (staging_rect, playfield_rect) =
        (&view_settings.staging_rect, &view_settings.playfield_rect);
    // draw the locked blocks
    for (y, slots_x) in playfield.slots.iter().enumerate() {
        let clearing = playfield
            .line_clear
            .as_ref()
            .filter(|line_clear| line_clear.lines.contains(&y));
        for (x, slot) in slots_x.iter().enumerate() {
            if let SlotState::Locked(rtype, wear) = slot {
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
                    if *wear == Wear::Tough {
                        // tough blocks survive the clear: they flash, but stay
                        color = block_color(*rtype, game_over, y as i32);
                        color = mix_color(color, Color::WHITE, flash);
                    }
                }
                let rect = playfield_block_rect([x as i32, y as i32], staging_rect, playfield_rect);
                let seed = y * playfield::PLAYFIELD_SLOTS[0] + x;
                if *rtype == RustominoType::Glitch && clearing.is_none() {
                    let calm = calm * GLITCH_LOCKED;
                    draw_glitch_block(canvas, meshes, rect, color, style, time, seed, calm);
                } else if *wear == Wear::Tough && game_over.is_none() {
                    // drawn with its piece below
                } else {
                    let crack = crack_color(scene.background.style, color, time, seed);
                    draw_block(canvas, meshes, rect, color, style);
                    draw_wear(canvas, meshes, rect, color, *wear, crack);
                }
            }
        }
    }

    // tough blocks show the scene through them, a piece at a time
    if game_over.is_none() {
        for piece in tough_pieces(playfield) {
            let blocks: Vec<(i32, i32, Rect)> = piece
                .iter()
                .map(|&(x, y)| {
                    let rect =
                        playfield_block_rect([x as i32, y as i32], staging_rect, playfield_rect);
                    (x as i32, y as i32, rect)
                })
                .collect();
            let (faces, bounds) = piece_faces(&blocks);
            for (&(x, y), (face, lowest)) in piece.iter().zip(faces) {
                let SlotState::Locked(rtype, _) = playfield.slots[y][x] else {
                    continue;
                };
                // icicles only hang over open space
                let open_below = y > 0 && playfield.slots[y - 1][x] == SlotState::Empty;
                let seed = y * playfield::PLAYFIELD_SLOTS[0] + x;
                let color = rtype.color();
                draw_tough(
                    canvas,
                    meshes,
                    scene,
                    face,
                    bounds,
                    color,
                    lowest && open_below,
                    seed,
                );
                // a line clear flashes it, but it stays
                let flash = playfield
                    .line_clear
                    .as_ref()
                    .filter(|line_clear| line_clear.lines.contains(&y))
                    .map_or(0.0, |line_clear| {
                        (1.0 - line_clear.progress() as f32 * 4.0).max(0.0)
                    });
                if flash > 0.0 {
                    draw_rect(canvas, meshes, face, with_alpha(Color::WHITE, flash));
                }
            }
        }
    }

    // draw the active rustomino over the locked blocks
    if let Some(active) = playfield
        .active_rustomino
        .as_ref()
        .filter(|active| active.tough && game_over.is_none())
    {
        let blocks: Vec<(i32, i32, Rect)> = active
            .playfield_slots()
            .iter()
            .map(|b| {
                (
                    b.x,
                    b.y,
                    playfield_block_rect([b.x, b.y], staging_rect, playfield_rect),
                )
            })
            .collect();
        let (faces, bounds) = piece_faces(&blocks);
        for (i, (&(x, y, _), (face, lowest))) in blocks.iter().zip(faces).enumerate() {
            let open_below = y > 0
                && playfield
                    .slots
                    .get(y as usize - 1)
                    .is_some_and(|row| row[x as usize] == SlotState::Empty);
            let color = active.rtype.color();
            draw_tough(
                canvas,
                meshes,
                scene,
                face,
                bounds,
                color,
                lowest && open_below,
                3000 + i,
            );
        }
    } else if let Some(active) = &playfield.active_rustomino {
        for (i, block) in active.playfield_slots().into_iter().enumerate() {
            let rect = playfield_block_rect([block[0], block[1]], staging_rect, playfield_rect);
            let color = block_color(active.rtype, game_over, block[1]);
            if active.rtype == RustominoType::Glitch {
                let seed = 1000 + i;
                draw_glitch_block(canvas, meshes, rect, color, style, time, seed, calm);
            } else {
                draw_block(canvas, meshes, rect, color, style);
                if active.tough {
                    draw_wear(canvas, meshes, rect, color, Wear::Tough, color);
                }
            }
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
        if ghost.rtype == RustominoType::Glitch && game_over.is_none() {
            // the outline split into magenta and cyan, jumping apart in bursts
            let frame = (time * GLITCH_RATE).floor();
            let burst = hash(7, frame * 0.731) < GLITCH_BURST;
            let split = snap(GLITCH_SPLIT * if burst { 2.0 } else { 1.0 });
            let alpha = GHOST_OUTLINE_ALPHA;
            for rect in ghost_outline(&slots, staging_rect, playfield_rect, GHOST_OUTLINE_WIDTH) {
                let shifted = |dx: f32| Rect::new(rect.x + dx, rect.y, rect.w, rect.h);
                draw_rect(
                    canvas,
                    meshes,
                    shifted(-split),
                    with_alpha(GLITCH_MAGENTA, alpha),
                );
                draw_rect(
                    canvas,
                    meshes,
                    shifted(split),
                    with_alpha(GLITCH_CYAN, alpha),
                );
            }
        } else if ghost.tough && game_over.is_none() {
            // a heavy outline, twice as thick and stronger, for a piece that lands hard
            let outline = with_alpha(color, (GHOST_OUTLINE_ALPHA * 1.4).min(1.0));
            let width = GHOST_OUTLINE_WIDTH * 2.0;
            for rect in ghost_outline(&slots, staging_rect, playfield_rect, width) {
                draw_rect(canvas, meshes, rect, outline);
            }
        } else {
            let outline = with_alpha(color, GHOST_OUTLINE_ALPHA);
            for rect in ghost_outline(&slots, staging_rect, playfield_rect, GHOST_OUTLINE_WIDTH) {
                draw_rect(canvas, meshes, rect, outline);
            }
        }
    }
}

/// fixed width outline inside the ghost piece's boundary, as non-overlapping rects
pub(super) fn ghost_outline(
    slots: &[IVec2],
    staging_rect: &Rect,
    playfield_rect: &Rect,
    width: f32,
) -> Vec<Rect> {
    let s = block_size();
    let (p, w) = (CELL - s, snap(width));
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

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_preview(
    canvas: &mut Painter,
    meshes: &Meshes,
    rustomino: &Option<Rustomino>,
    preview_rect: &Rect,
    game_over: Option<&GameOver>,
    style: BlockStyle,
    time: f32,
    scene: &ToughScene,
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
        let color = block_color(rustomino.rtype, game_over, PREVIEW_ROW);
        if rustomino.tough && game_over.is_none() {
            // the scene through the whole piece, as on the board
            let blocks: Vec<(i32, i32, Rect)> = rustomino
                .blocks
                .iter()
                .map(|block| {
                    let rect = Rect::new(
                        left + (block.x - min.x) as f32 * cell,
                        top + (max.y - block.y) as f32 * cell,
                        block_size(),
                        block_size(),
                    );
                    (block.x, block.y, rect)
                })
                .collect();
            let (faces, bounds) = piece_faces(&blocks);
            for (i, (face, _)) in faces.into_iter().enumerate() {
                let seed = 4000 + i + preview_rect.x as usize;
                draw_tough(canvas, meshes, scene, face, bounds, color, false, seed);
            }
            return;
        }
        for (i, block) in rustomino.blocks.iter().enumerate() {
            let rect = Rect::new(
                left + (block.x - min.x) as f32 * cell,
                top + (max.y - block.y) as f32 * cell,
                block_size(),
                block_size(),
            );
            if rustomino.rtype == RustominoType::Glitch {
                // seeded by the panel so next and hold glitch apart
                let seed = 2000 + i + preview_rect.x as usize;
                let calm = glitch_calm(game_over, scene);
                draw_glitch_block(canvas, meshes, rect, color, style, time, seed, calm);
            } else {
                draw_block(canvas, meshes, rect, color, style);
                if rustomino.tough {
                    draw_wear(canvas, meshes, rect, color, Wear::Tough, color);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_playing(
    canvas: &mut Painter,
    meshes: &Meshes,
    playfield: &Playfield,
    next_rustomino: &Option<Rustomino>,
    hold_rustomino: &Option<Rustomino>,
    view_settings: &ViewSettings,
    game_over: Option<&GameOver>,
    style: &Style,
    assists: bool, // ghost and hold
    time: f32,     // seconds, drives the glitch pieces
    scene: &ToughScene,
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
        time,
        scene,
    );
    if assists {
        draw_preview(
            canvas,
            meshes,
            hold_rustomino,
            &view_settings.hold_rect,
            game_over,
            style.block,
            time,
            scene,
        );
    }
    draw_preview(
        canvas,
        meshes,
        next_rustomino,
        &view_settings.preview_rect,
        game_over,
        style.block,
        time,
        scene,
    );
}

pub(super) fn draw_playing_text(
    canvas: &mut Painter,
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
pub(super) fn draw_counter(canvas: &mut Painter, score: &ScoreDisplay, pos: Vec2) {
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
    canvas: &mut Painter,
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
    canvas: &mut Painter,
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
    for (n, trail) in effects.trails.iter().enumerate() {
        let fade = trail.fade();
        let x = left + trail.x * cell;
        let length = trail.from - trail.to;
        for i in 0..TRAIL_SEGMENTS {
            let t = i as f32 / TRAIL_SEGMENTS as f32;
            let y0 = trail.to + length * t;
            let h = length / TRAIL_SEGMENTS as f32 * cell;
            let rect = snap_edges(Rect::new(x, bottom - y0 * cell - h, cell, h));
            let alpha = TRAIL_ALPHA * trail.strength * fade * (1.0 - t);
            if trail.style == DropStyle::Glitch {
                // split into magenta and cyan, segments dropping out as it flickers
                if hash(n * 31 + i, (fade * 24.0).floor()) < 0.3 {
                    continue;
                }
                let split = snap(GLITCH_SPLIT * 1.5);
                let shifted = |dx: f32| Rect::new(rect.x + dx, rect.y, rect.w, rect.h);
                draw_rect(
                    canvas,
                    meshes,
                    shifted(-split),
                    with_alpha(GLITCH_MAGENTA, alpha),
                );
                draw_rect(
                    canvas,
                    meshes,
                    shifted(split),
                    with_alpha(GLITCH_CYAN, alpha),
                );
            } else {
                draw_rect(canvas, meshes, rect, with_alpha(trail.color, alpha));
            }
        }
    }
    if let Some(impact) = &effects.impact {
        let alpha = (IMPACT_ALPHA * impact.strength * impact.fade()).min(1.0);
        for block in impact.blocks {
            let rect =
                playfield_block_rect([block.x, block.y], &view_settings.staging_rect, &playfield);
            if impact.style == DropStyle::Glitch {
                // a split flash, magenta and cyan to either side
                let split = snap(GLITCH_SPLIT * 2.0);
                let shifted = |dx: f32| Rect::new(rect.x + dx, rect.y, rect.w, rect.h);
                draw_rect(
                    canvas,
                    meshes,
                    shifted(-split),
                    with_alpha(GLITCH_MAGENTA, alpha),
                );
                draw_rect(
                    canvas,
                    meshes,
                    shifted(split),
                    with_alpha(GLITCH_CYAN, alpha),
                );
            } else {
                draw_rect(canvas, meshes, rect, with_alpha(Color::WHITE, alpha));
            }
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
    pub shake: Option<Vec2>,        // None when not shaking
    pub glitch_time: Option<f32>,   // fixed clock for glitch pieces, None runs with the game
    pub background: &'a Background, // tough blocks show its scene
}

impl Layer for GameLayer<'_> {
    fn draw(&self, canvas: &mut Painter, frame: &Frame) -> GameResult {
        let screen = canvas.screen_coordinates();
        if let (Some(rect), Some(shake)) = (screen, self.shake) {
            set_screen_coordinates(
                canvas,
                Rect::new(rect.x - shake.x, rect.y - shake.y, rect.w, rect.h),
            );
        }
        let time = self
            .glitch_time
            .unwrap_or_else(|| frame.ctx.time.time_since_start().as_secs_f32());
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
            time,
            &ToughScene {
                background: self.background,
                board: frame.view.playfield_rect,
                time,
            },
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
