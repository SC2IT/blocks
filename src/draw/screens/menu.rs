use ggez::glam::Vec2;
use ggez::graphics::Rect;
use ggez::GameResult;

use crate::menus::{self, Menu};

use super::*;
use crate::draw::*;

/// main menu
pub struct MenuScreen<'a> {
    pub menu: &'a menus::MenuList,
}

impl Layer for MenuScreen<'_> {
    fn draw(&self, canvas: &mut Painter, frame: &Frame) -> GameResult {
        let Self { menu } = *self;
        let ctx = frame.ctx;
        let meshes = frame.meshes;
        let view_settings = frame.view;
        let style = frame.block;
        let view_rect = &view_settings.view_rect;
        draw_name_title(
            ctx,
            canvas,
            view_rect.h * 0.16,
            TITLE_SIZE,
            view_rect,
            frame.glitch_time,
            frame.low_sensory,
        )?;
        if frame.low_sensory {
            draw_low_sensory_badge(ctx, canvas, meshes, view_rect)?;
        }
        let panel = menu_panel_rect(
            view_rect,
            menu.items().len(),
            view_rect.h * 0.42,
            MENU_PANEL_WIDTH,
            MENU_ROW_HEIGHT,
        );
        draw_panel(canvas, meshes, panel);
        draw_menu_list(ctx, canvas, meshes, menu, panel, style)?;
        draw_hint(ctx, canvas, "Up/Down: select   Enter: choose", view_rect)
    }
}

const BADGE_WIDTH: f32 = 440.0;
const BADGE_HEIGHT: f32 = 62.0;

/// low sensory is on, and how to turn it off
fn draw_low_sensory_badge(
    ctx: &ggez::Context,
    canvas: &mut Painter,
    meshes: &Meshes,
    view_rect: &Rect,
) -> GameResult {
    let center_y = view_rect.h * 0.31;
    let badge = Rect::new(
        snap(view_rect.w / 2.0 - BADGE_WIDTH / 2.0),
        snap(center_y - BADGE_HEIGHT / 2.0),
        BADGE_WIDTH,
        BADGE_HEIGHT,
    );
    draw_panel(canvas, meshes, badge);
    let x = view_rect.w / 2.0;
    draw_text_centered(
        ctx,
        canvas,
        &ui_text("Low Sensory On", UI_FONT_SIZE),
        Vec2::new(x, center_y - 12.0),
        ACCENT_COLOR,
    )?;
    draw_text_centered(
        ctx,
        canvas,
        &ui_text("Turn off: Options > Low Sensory", SMALL_TEXT_SIZE),
        Vec2::new(x, center_y + 16.0),
        LABEL_COLOR,
    )
}
