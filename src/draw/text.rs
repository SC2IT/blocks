use ggez::glam::Vec2;
use ggez::graphics::{self, Color, Rect, TextFragment};
use ggez::{Context, GameResult};

use crate::rustomino::RustominoType;
use crate::util;
use std::sync::atomic::{AtomicU32, Ordering};
use strum::IntoEnumIterator;

use super::*;

// physical pixels per view unit, text is rasterized at the physical size
pub(super) static PIXEL_SCALE: AtomicU32 = AtomicU32::new(1.0f32.to_bits());

pub fn set_pixel_scale(scale: f32) {
    PIXEL_SCALE.store(scale.to_bits(), Ordering::Relaxed);
}

pub(super) fn pixel_scale() -> f32 {
    f32::from_bits(PIXEL_SCALE.load(Ordering::Relaxed))
}

pub(super) fn px(size: f32) -> graphics::PxScale {
    graphics::PxScale::from(size * pixel_scale())
}

pub(super) fn text_param() -> graphics::DrawParam {
    graphics::DrawParam::default().scale(Vec2::splat(1.0 / pixel_scale()))
}

pub(super) trait MeasureUi {
    /// size in view units
    fn measure_ui(&self, ctx: &Context) -> GameResult<Vec2>;
}

impl MeasureUi for graphics::Text {
    fn measure_ui(&self, ctx: &Context) -> GameResult<Vec2> {
        Ok(Vec2::from(self.measure(ctx)?) / pixel_scale())
    }
}

pub(super) fn ui_text(text: &str, size: f32) -> graphics::Text {
    let mut text = graphics::Text::new(text);
    text.set_font("04b30").set_scale(px(size));
    text
}

pub(super) fn draw_shadowed(canvas: &mut Painter, text: &graphics::Text, pos: Vec2, color: Color) {
    let param = text_param();
    // the shadow fades with the text
    let shadow = with_alpha(TEXT_SHADOW_COLOR, TEXT_SHADOW_COLOR.a * color.a);
    draw(
        canvas,
        text,
        param.dest(pos + Vec2::new(2.0, 2.0)).color(shadow),
    );
    draw(canvas, text, param.dest(pos).color(premultiply(color)));
}

/// text in the ui font with a drop shadow
pub(super) fn draw_text(canvas: &mut Painter, text: &str, pos: Vec2, size: f32, color: Color) {
    draw_shadowed(canvas, &ui_text(text, size), pos, color);
}

/// text positioned by its center
pub(super) fn draw_text_centered(
    ctx: &Context,
    canvas: &mut Painter,
    text: &graphics::Text,
    center: Vec2,
    color: Color,
) -> GameResult {
    let size = text.measure_ui(ctx)?;
    draw_shadowed(
        canvas,
        text,
        center - Vec2::new(size.x / 2.0, size.y / 2.0),
        color,
    );
    Ok(())
}

/// large wobbling title, optionally with each letter in a piece color
/// text with each letter colored like a piece, and a matching shadow text
/// per letter colors override the draw color, so the shadow is its own text
pub(super) fn piece_colored_text(text: &str, size: f32) -> (graphics::Text, graphics::Text) {
    piece_colored_text_alpha(text, size, 1.0)
}

pub(super) fn piece_colored_text_alpha(
    text: &str,
    size: f32,
    alpha: f32,
) -> (graphics::Text, graphics::Text) {
    let colors: Vec<Color> = RustominoType::iter()
        .map(|t| with_alpha(t.color(), alpha))
        .collect();
    let letters = |color_at: &dyn Fn(usize) -> Color| {
        let mut letters = graphics::Text::default();
        for (i, c) in text.chars().enumerate() {
            letters.add(TextFragment::new(c).color(color_at(i)));
        }
        letters.set_font("04b30").set_scale(px(size));
        letters
    };
    (
        letters(&|i| premultiply(tint(colors[i % colors.len()], TITLE_TINT))),
        letters(&|_| with_alpha(TEXT_SHADOW_COLOR, TEXT_SHADOW_COLOR.a * alpha)),
    )
}

pub(super) fn draw_piece_colored(canvas: &mut Painter, text: &str, pos: Vec2, size: f32) {
    let (text, shadow) = piece_colored_text(text, size);
    let param = text_param();
    draw(
        canvas,
        &shadow,
        param.dest(pos + Vec2::splat((size / 16.0).max(2.0))),
    );
    draw(canvas, &text, param.dest(pos));
}

/// large wobbling piece colored title
pub(super) fn draw_title(
    ctx: &Context,
    canvas: &mut Painter,
    title: &str,
    center_y: f32,
    size: f32,
    view_rect: &Rect,
) -> GameResult {
    let wobble = util::slow_wobble(ctx.time.time_since_start().as_secs_f32());
    let center = Vec2::new(view_rect.w / 2.0, center_y + wobble * size / 12.0);
    let dims = piece_colored_text(title, size).0.measure_ui(ctx)?;
    let pos = center - Vec2::new(dims.x / 2.0, dims.y / 2.0);
    draw_piece_colored(canvas, title, pos, size);
    Ok(())
}

/// the game's name, as the title on the menu and the board
pub const GAME_NAME: &str = "Bitrot";
const NAME_GLITCH_EVERY: f32 = 2.4; // seconds between chances of a glitch
const NAME_GLITCH_CHANCE: f32 = 0.6;
const NAME_GLITCH_TIME: f32 = 0.32; // seconds a glitch lasts
const NAME_GLITCH_RATE: f32 = 18.0; // jitters a second during a glitch
const NAME_GLITCH_SPLIT: f32 = 0.07; // magenta and cyan apart, of the text size
const NAME_GLITCH_JUMP: f32 = 0.1; // how far a letter jumps, of the text size
const NAME_GLITCH_ALPHA: f32 = 0.9;
const NAME_FROZEN: [(usize, f32); 2] = [(2, 0.6), (4, -0.4)]; // low sensory: T and O held mid glitch, (letter, jump)

/// the name's letters glitching at a moment: (letter, sideways jump in text sizes, split in text sizes)
fn name_glitches(letters: usize, time: f32, still: bool) -> Vec<(usize, f32, f32)> {
    if still {
        return NAME_FROZEN
            .iter()
            .filter(|(i, _)| *i < letters)
            .map(|&(i, jump)| (i, jump * NAME_GLITCH_JUMP, NAME_GLITCH_SPLIT))
            .collect();
    }
    let slot = (time / NAME_GLITCH_EVERY).floor().max(0.0) as usize;
    if hash(slot, 81.0) > NAME_GLITCH_CHANCE {
        return Vec::new();
    }
    let start =
        slot as f32 * NAME_GLITCH_EVERY + hash(slot, 82.0) * (NAME_GLITCH_EVERY - NAME_GLITCH_TIME);
    if !(0.0..NAME_GLITCH_TIME).contains(&(time - start)) {
        return Vec::new();
    }
    // one letter, sometimes two, jittering while it lasts
    let beat = (time * NAME_GLITCH_RATE).floor();
    let count = if hash(slot, 83.0) < 0.35 { 2 } else { 1 };
    (0..count)
        .map(|k| {
            // the second letter is never the first
            let first = (hash(slot, 84.0) * letters as f32) as usize % letters;
            let other = (hash(slot, 85.0) * (letters - 1) as f32) as usize % (letters - 1).max(1);
            let i = if k == 0 {
                first
            } else {
                (first + 1 + other) % letters
            };
            let n = |j: f32| hash(slot * 7 + i, beat * 0.37 + j);
            let jump = (n(1.0) * 2.0 - 1.0) * NAME_GLITCH_JUMP;
            let split = NAME_GLITCH_SPLIT * (0.6 + 0.8 * n(2.0));
            (i, jump, split)
        })
        .collect()
}

/// the game's name in piece colors, letters glitching now and then, or two held mid glitch when still
pub(super) fn draw_game_name(canvas: &mut Painter, pos: Vec2, size: f32, time: f32, still: bool) {
    let name = GAME_NAME;
    let letters = name.chars().count();
    let glitches = name_glitches(letters, time, still);
    if glitches.is_empty() {
        draw_piece_colored(canvas, name, pos, size);
        return;
    }
    let colors: Vec<Color> = RustominoType::iter().map(|t| t.color()).collect();
    let color_of = |i: usize| premultiply(tint(colors[i % colors.len()], TITLE_TINT));
    // per letter colors, clear letters keep the others in place
    let text = |color_at: &dyn Fn(usize) -> Color| {
        let mut text = graphics::Text::default();
        for (i, c) in name.chars().enumerate() {
            text.add(TextFragment::new(c).color(color_at(i)));
        }
        text.set_font("04b30").set_scale(px(size));
        text
    };
    let glitching = |i: usize| glitches.iter().any(|g| g.0 == i);
    let clear = Color::new(0.0, 0.0, 0.0, 0.0);
    let param = text_param();
    let shadow = text(&|i| {
        if glitching(i) {
            clear
        } else {
            TEXT_SHADOW_COLOR
        }
    });
    draw(
        canvas,
        &shadow,
        param.dest(pos + Vec2::splat((size / 16.0).max(2.0))),
    );
    let steady = text(&|i| if glitching(i) { clear } else { color_of(i) });
    draw(canvas, &steady, param.dest(pos));
    for &(g, jump, split) in &glitches {
        let only = |color: Color| text(&|i| if i == g { color } else { clear });
        let at = pos + Vec2::new(snap(jump * size), 0.0);
        let apart = Vec2::new(snap(split * size).max(2.0), 0.0);
        let magenta = premultiply(with_alpha(super::board::GLITCH_MAGENTA, NAME_GLITCH_ALPHA));
        let cyan = premultiply(with_alpha(super::board::GLITCH_CYAN, NAME_GLITCH_ALPHA));
        draw(canvas, &only(magenta), param.dest(at - apart));
        draw(canvas, &only(cyan), param.dest(at + apart));
        draw(canvas, &only(color_of(g)), param.dest(at));
    }
}

/// the game's name as a large wobbling title
pub(super) fn draw_name_title(
    ctx: &Context,
    canvas: &mut Painter,
    center_y: f32,
    size: f32,
    view_rect: &Rect,
    time: f32,
    still: bool,
) -> GameResult {
    let wobble = if still {
        0.0
    } else {
        util::slow_wobble(ctx.time.time_since_start().as_secs_f32())
    };
    let center = Vec2::new(view_rect.w / 2.0, center_y + wobble * size / 12.0);
    let dims = piece_colored_text(GAME_NAME, size).0.measure_ui(ctx)?;
    let pos = center - Vec2::new(dims.x / 2.0, dims.y / 2.0);
    draw_game_name(canvas, pos, size, time, still);
    Ok(())
}

pub(super) fn draw_hint(
    ctx: &Context,
    canvas: &mut Painter,
    hint: &str,
    view_rect: &Rect,
) -> GameResult {
    let center = Vec2::new(view_rect.w / 2.0, view_rect.h - 32.0);
    draw_text_centered(
        ctx,
        canvas,
        &ui_text(hint, SMALL_TEXT_SIZE),
        center,
        LABEL_COLOR,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_glitches_now_and_then() {
        let moments: Vec<_> = (0..1200)
            .map(|i| name_glitches(6, i as f32 * 0.05, false))
            .collect();
        let glitching = moments.iter().filter(|g| !g.is_empty()).count();
        // now and then, not all the time
        assert!(glitching > 0 && glitching < moments.len() / 4);
        for glitches in &moments {
            assert!(glitches.iter().all(|g| g.0 < 6));
            if let [a, b] = glitches.as_slice() {
                assert_ne!(a.0, b.0);
            }
        }
    }

    #[test]
    fn low_sensory_holds_two_letters() {
        let held = name_glitches(6, 0.0, true);
        assert_eq!(held.len(), 2);
        assert_eq!(held, name_glitches(6, 17.3, true));
    }
}
