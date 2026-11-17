// tough blocks: the background's own scene shows through them, filling the whole block
// with no outline, so a tough piece reads as a window into the background. everything runs
// on the background's clocks, so it speeds up and builds with the level like the background

use ggez::graphics::{Color, Rect};

use crate::background::{Background, BackgroundStyle};
use crate::rustomino::RustominoType;
use strum::IntoEnumIterator;

use super::background::{
    sea_wave, storm_strike, CREST_COLOR, FLASH_COLOR, SEA_LAYERS, WHITEOUT_COLOR,
};
use super::*;

// stars: the star flight seen through the piece, under a steady glow of the piece's color
const STARS_DARK: Color = Color::new(0.03, 0.03, 0.09, 1.0);
const STARS_COUNT: usize = 500;
const STARS_FLIGHT: f32 = 0.1; // depth per second of background time
const STARS_GLOW: f32 = 0.16; // the steady nebula hue
const STARS_GLAZE: (f32, f32) = (0.06, 0.08); // lighter top left, darker bottom right of the piece
const STAR_COLOR: Color = Color::new(0.8, 0.9, 1.0, 1.0);

// plasma: the background's plasma in a grid, six squares a block, unbroken across the piece
const PLASMA_DARK: Color = Color::new(0.04, 0.02, 0.08, 1.0);
const PLASMA_SQUARES: f32 = 6.0; // per block
const PLASMA_SPEED: f32 = 0.8;

// storm: sky, rain and lightning above a sea filling the piece's bottom block and a third
// of the one above it, under a thin frost
const STORM_SKY: Color = Color::new(0.04, 0.05, 0.08, 1.0);
const STORM_SEA_RISE: f32 = 1.3; // blocks up from the piece's bottom edge
const STORM_SEA_SINGLE: f32 = 0.45; // waterline down a one row piece
const STORM_WAVE: f32 = 0.18; // wave height per sea layer height, in px
const STORM_STRIP: f32 = 2.0; // px per wave column
const STORM_RAIN: (f32, f32) = (3.0, 8.0); // drops per block, calm to full storm
const STORM_RAIN_COLOR: Color = Color::new(0.6, 0.7, 0.85, 0.7);
const STORM_FROST: f32 = 0.05;
const ICE_COLOR: Color = Color::new(0.85, 0.94, 1.0, 0.9);

// off: calm, random static with scanlines, tinted by the piece
const OFF_DARK: Color = Color::new(0.11, 0.08, 0.19, 1.0);
const OFF_GRAIN: f32 = 3.75; // px per static cell, eight to a block
const OFF_RATE: f32 = 9.0; // static changes a second, on uneven beats
const OFF_STATIC: f32 = 0.58;
const OFF_TINT: f32 = 0.25;
const OFF_BAND_EVERY: f32 = 2.5; // seconds between chances of a rolling band

/// what a tough block shows: the background's scene, laid over the playfield
pub(super) struct ToughScene<'a> {
    pub background: &'a Background,
    pub board: Rect, // the playfield, the scene's frame
    pub time: f32,   // seconds, for the static, which the game's still off background lacks
}

/// draw r cut down to clip, if any of it is left
fn clipped(canvas: &mut Painter, meshes: &Meshes, r: Rect, clip: &Rect, color: Color) {
    let x0 = r.x.max(clip.x);
    let y0 = r.y.max(clip.y);
    let x1 = (r.x + r.w).min(clip.x + clip.w);
    let y1 = (r.y + r.h).min(clip.y + clip.h);
    if x1 > x0 && y1 > y0 {
        draw_rect(canvas, meshes, Rect::new(x0, y0, x1 - x0, y1 - y0), color);
    }
}

/// a tough block's face: `block` is the block, grown over the gaps to its piece mates,
/// `piece` the whole piece's bounds, `bottom` is set when nothing of the piece is below it
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_tough(
    canvas: &mut Painter,
    meshes: &Meshes,
    scene: &ToughScene,
    block: Rect,
    piece: Rect,
    color: Color,
    bottom: bool,
    seed: usize,
) {
    let alpha = color.a;
    match scene.background.style {
        BackgroundStyle::Stars => stars(canvas, meshes, scene, block, piece, color),
        BackgroundStyle::Plasma => plasma(canvas, meshes, scene, block),
        BackgroundStyle::Storm => {
            storm(canvas, meshes, scene, block, piece, seed);
            if bottom {
                icicle(canvas, meshes, block, seed, scene.background.sea);
            }
        }
        BackgroundStyle::Off => off(canvas, meshes, scene, block, color, seed),
    }
    // the game over curtain and line clears fade the block
    if alpha < 1.0 {
        draw_rect(canvas, meshes, block, with_alpha(Color::BLACK, 1.0 - alpha));
    }
}

fn stars(
    canvas: &mut Painter,
    meshes: &Meshes,
    scene: &ToughScene,
    block: Rect,
    piece: Rect,
    color: Color,
) {
    let background = scene.background;
    draw_rect(canvas, meshes, block, STARS_DARK);
    draw_rect(canvas, meshes, block, with_alpha(color, STARS_GLOW));
    // stars flying out from the middle of the board, bigger and brighter as they come near
    let board = scene.board;
    let center = (board.x + board.w / 2.0, board.y + board.h / 2.0);
    let field = background.stars as f32 * 10.0;
    for i in 0..STARS_COUNT {
        let n = |k: f32| hash(i, k + field);
        let z = (n(1.0) - background.time * STARS_FLIGHT)
            .rem_euclid(1.0)
            .max(0.02);
        let depth = 1.0 - z;
        let x = center.0 + (n(2.0) * 2.0 - 1.0) * 0.5 / z * board.w * 0.6;
        let y = center.1 + (n(3.0) * 2.0 - 1.0) * 0.5 / z * board.h * 0.6;
        if x < block.x - 3.0 || x > block.right() || y < block.y - 3.0 || y > block.bottom() {
            continue;
        }
        let size = (1.0 + 2.0 * depth * depth).round();
        let twinkle = 0.6
            + 0.4
                * (background.twinkle * (2.0 + 3.0 * n(4.0)) + n(5.0) * std::f32::consts::TAU)
                    .sin();
        let arrive = super::background::smoothstep(1.0, 0.8, z);
        let a = ((0.45 + 0.75 * depth) * twinkle).min(1.0) * arrive;
        let star = if n(6.0) < 0.15 {
            RustominoType::iter()
                .nth(i % 7)
                .map_or(STAR_COLOR, |t| t.color())
        } else {
            STAR_COLOR
        };
        let r = Rect::new(snap(x - size / 2.0), snap(y - size / 2.0), size, size);
        clipped(canvas, meshes, r, &block, with_alpha(star, a));
    }
    // a very slight light-to-dark glaze across the whole piece, so it reads as one shape
    let t = ((block.center().x - piece.x) / piece.w + (block.center().y - piece.y) / piece.h) / 2.0;
    let glaze = if t < 0.5 {
        with_alpha(Color::WHITE, STARS_GLAZE.0 * (1.0 - 2.0 * t))
    } else {
        with_alpha(Color::BLACK, STARS_GLAZE.1 * (2.0 * t - 1.0))
    };
    draw_rect(canvas, meshes, block, glaze);
}

fn plasma(canvas: &mut Painter, meshes: &Meshes, scene: &ToughScene, block: Rect) {
    draw_rect(canvas, meshes, block, PLASMA_DARK);
    let colors: Vec<Color> = RustominoType::iter().map(|t| t.color()).collect();
    let board = scene.board;
    let (cx, cy) = (board.x + board.w / 2.0, board.y + board.h / 2.0);
    let t = scene.background.time * PLASMA_SPEED;
    // squares on the block grid, so the grid runs on unbroken across the piece
    let pitch = CELL / PLASMA_SQUARES;
    let mut y = block.y;
    while y < block.bottom() {
        let mut x = block.x;
        while x < block.right() {
            let px = (x + pitch / 2.0 - cx) / 12.0;
            let py = (y + pitch / 2.0 - cy) / 12.0;
            let ring = (px * px + py * py).sqrt();
            let v = (px * 0.35 + t * 1.1).sin()
                + (py * 0.3 - t * 0.8).sin()
                + ((px + py) * 0.18 + t * 0.6).sin()
                + (ring * 0.4 - t * 1.5).sin();
            let n = (v + 4.0) / 8.0;
            let color = colors[(n * 10.0 + t * 0.3) as usize % colors.len()];
            let color = if n > 0.62 {
                tint(color, (n - 0.62) * 0.9)
            } else {
                with_alpha(color, 0.55 + 0.45 * n / 0.62)
            };
            let square = Rect::new(snap(x), snap(y), snap(pitch) - 1.0, snap(pitch) - 1.0);
            clipped(canvas, meshes, square, &block, color);
            x += pitch;
        }
        y += pitch;
    }
}

fn storm(
    canvas: &mut Painter,
    meshes: &Meshes,
    scene: &ToughScene,
    block: Rect,
    piece: Rect,
    seed: usize,
) {
    let background = scene.background;
    draw_rect(canvas, meshes, block, STORM_SKY);
    // rain, heavier and more slanted as the storm builds
    let storm = background.storm;
    let drops = (STORM_RAIN.0 + (STORM_RAIN.1 - STORM_RAIN.0) * storm) as usize;
    let slant = 0.18 + 0.57 * storm;
    for j in 0..drops {
        let n = |k: f32| hash(seed * 31 + j, k);
        let phase = (n(1.0) + background.rain * (1.0 + 0.5 * n(3.0))).fract();
        let y = block.y - 6.0 + phase * (block.h + 6.0);
        let x = block.x + n(2.0) * (block.w + 6.0) - phase * slant * 6.0;
        for step in 0..3 {
            let s = step as f32;
            let drop = Rect::new(snap(x - s * slant * 2.0), snap(y + s * 2.0), 1.0, 2.0);
            clipped(canvas, meshes, drop, &block, STORM_RAIN_COLOR);
        }
    }
    // the sea, its waterline set by the piece, so every piece has its own
    let sea = if piece.h > CELL * 1.5 {
        piece.bottom() - CELL * STORM_SEA_RISE
    } else {
        piece.y + piece.h * STORM_SEA_SINGLE
    };
    let heave = 1.0 + 1.4 * background.sea_storm;
    for (layer, (color, depth, height)) in SEA_LAYERS.iter().enumerate() {
        let base = sea + depth * (piece.bottom() - sea);
        let mut x = (block.x / STORM_STRIP).floor() * STORM_STRIP;
        while x < block.right() {
            let wave = sea_wave(x * 3.0, layer, background.sea) * height * STORM_WAVE * heave;
            let top = snap(base - wave);
            let water = Rect::new(x, top, STORM_STRIP, block.bottom() - top);
            clipped(canvas, meshes, water, &block, *color);
            let crest = Rect::new(x, top, STORM_STRIP, 1.0);
            clipped(canvas, meshes, crest, &block, with_alpha(CREST_COLOR, 0.8));
            x += STORM_STRIP;
        }
    }
    // a flash when lightning strikes, then frost and the whiteout over it all
    if let Some((_, flash)) = storm_strike(background.bolt) {
        draw_rect(canvas, meshes, block, with_alpha(FLASH_COLOR, 0.35 * flash));
    }
    draw_rect(canvas, meshes, block, with_alpha(Color::WHITE, STORM_FROST));
    if background.whiteout > 0.0 {
        draw_rect(
            canvas,
            meshes,
            block,
            with_alpha(WHITEOUT_COLOR, 0.55 * background.whiteout),
        );
    }
}

/// an icicle under a storm block, with a drop now and then
fn icicle(canvas: &mut Painter, meshes: &Meshes, block: Rect, seed: usize, clock: f32) {
    if hash(seed, 21.0) >= 0.6 {
        return;
    }
    let width = 6.0;
    let length = (block.h * (0.3 + 0.3 * hash(seed, 22.0))).round();
    let x = snap(block.x + hash(seed, 23.0) * (block.w - width));
    let mut y = block.bottom();
    let mut w = width;
    while w >= 2.0 {
        let step = (length / 3.0).max(1.0);
        draw_rect(
            canvas,
            meshes,
            Rect::new(x + (width - w) / 2.0, y, w, step),
            ICE_COLOR,
        );
        y += step;
        w -= 2.0;
    }
    let p = (clock * 0.7 + hash(seed, 24.0) * 2.0) % 2.0;
    if p < 1.0 {
        let drop = Rect::new(x + width / 2.0 - 0.5, y + p * block.h * 0.6, 1.0, 2.0);
        draw_rect(canvas, meshes, drop, with_alpha(ICE_COLOR, 1.0 - p));
    }
}

fn off(
    canvas: &mut Painter,
    meshes: &Meshes,
    scene: &ToughScene,
    block: Rect,
    color: Color,
    seed: usize,
) {
    let t = scene.time;
    draw_rect(canvas, meshes, block, OFF_DARK);
    // static that changes on uneven beats, never holding still
    let beat = (t * OFF_RATE + 0.7 * (t * 2.3).sin()).floor();
    let cols = (block.w / OFF_GRAIN).ceil() as usize;
    let rows = (block.h / OFF_GRAIN).ceil() as usize;
    for row in 0..rows {
        // rows brighten and dim like interference
        let gain = 0.6 + 0.6 * hash(seed * 97 + row, beat * 0.37 + 5.0);
        for col in 0..cols {
            let n = hash(seed * 997 + row * 31 + col, beat * 0.731);
            let v = (n * n * 0.78 * gain).min(1.0);
            let cell = Rect::new(
                block.x + col as f32 * OFF_GRAIN,
                block.y + row as f32 * OFF_GRAIN,
                OFF_GRAIN,
                OFF_GRAIN,
            );
            clipped(
                canvas,
                meshes,
                cell,
                &block,
                Color::new(v, v, v, OFF_STATIC),
            );
        }
    }
    draw_rect(canvas, meshes, block, with_alpha(color, OFF_TINT));
    // scanlines, and now and then a band rolling down at its own pace
    let mut y = block.y;
    while y < block.bottom() {
        clipped(
            canvas,
            meshes,
            Rect::new(block.x, y, block.w, 1.0),
            &block,
            with_alpha(Color::BLACK, 0.35),
        );
        y += 3.0;
    }
    let window = (t / OFF_BAND_EVERY).floor();
    let into = t - window * OFF_BAND_EVERY;
    let k = window as usize;
    let duration = 1.2 + 2.2 * hash(k, 41.0);
    let p = into / duration;
    if hash(k, 42.0) < 0.65 && p < 1.0 {
        let board = scene.board;
        let height = board.h * (0.04 + 0.08 * hash(k, 43.0));
        let band = Rect::new(
            block.x,
            board.y + (p * 1.3 - 0.15) * board.h,
            block.w,
            height,
        );
        clipped(
            canvas,
            meshes,
            band,
            &block,
            with_alpha(color, 0.18 + 0.14 * hash(k, 44.0)),
        );
    }
}

/// a cracked block keeps a trace of the scene it showed: a crack in the scene's color
pub(super) fn crack_color(style: BackgroundStyle, color: Color, time: f32, seed: usize) -> Color {
    match style {
        // pulsing plasma in the crack
        BackgroundStyle::Plasma => {
            let pulse = 0.5 + 0.5 * (time * 4.0 + seed as f32).sin();
            mix_color(shade(color, 0.3), Color::new(1.0, 0.44, 0.85, 1.0), pulse)
        }
        // pale ice
        BackgroundStyle::Storm => mix_color(shade(color, 0.3), ICE_COLOR, 0.6),
        // starlight caught in the crack, now and then
        BackgroundStyle::Stars => {
            let on = (time * 3.0 + seed as f32).sin() > 0.7;
            if on {
                STAR_COLOR
            } else {
                mix_color(shade(color, 0.3), Color::new(0.75, 0.66, 1.0, 1.0), 0.35)
            }
        }
        // static still flickers along it
        BackgroundStyle::Off => {
            if hash(seed, (time * 6.0).floor()) < 0.25 {
                Color::new(1.0, 0.2, 0.75, 1.0)
            } else {
                shade(color, 0.3)
            }
        }
    }
}
