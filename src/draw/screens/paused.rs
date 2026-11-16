use ggez::graphics::Canvas;
use ggez::GameResult;

use crate::menus::{self, Menu};

use super::*;
use crate::draw::*;

/// pause menu, over the game
pub struct PausedScreen<'a> {
    pub menu: &'a menus::MenuList,
}

impl Layer for PausedScreen<'_> {
    fn draw(&self, canvas: &mut Canvas, frame: &Frame) -> GameResult {
        let Self { menu } = *self;
        let ctx = frame.ctx;
        let meshes = frame.meshes;
        let view_settings = frame.view;
        let style = frame.block;
        let view_rect = &view_settings.view_rect;
        draw_overlay(canvas, meshes, view_rect);
        let panel = menu_panel_rect(
            view_rect,
            menu.items().len(),
            view_rect.h * 0.36,
            MENU_PANEL_WIDTH,
            MENU_ROW_HEIGHT,
        );
        let title_y = panel.y - PAUSED_TITLE_SIZE * 0.75;
        draw_title(ctx, canvas, "Paused", title_y, PAUSED_TITLE_SIZE, view_rect)?;
        draw_panel(canvas, meshes, panel);
        draw_menu_list(ctx, canvas, meshes, menu, panel, style)?;
        draw_hint(
            ctx,
            canvas,
            "Up/Down: select   Enter: choose   Esc: resume",
            view_rect,
        )
    }
}
