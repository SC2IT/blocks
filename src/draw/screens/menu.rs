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
        draw_title(
            ctx,
            canvas,
            "Blocks!",
            view_rect.h * 0.16,
            TITLE_SIZE,
            view_rect,
        )?;
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
