mod buildings;
mod placement;
mod vista;

use super::*;
use macroquad_toolkit::ui::draw_ui_text;

impl SectBaseState {
    pub(super) fn draw_map_view(
        &mut self,
        rect: Rect,
        data: &mut GameData,
        textures: &TextureManager,
    ) -> Option<UpdateResult> {
        let viewport = self.sect_map_view_rect(rect);
        let ui = macroquad_toolkit::ui::VirtualUi::from_responsive_screen_size(
            screen_width(),
            screen_height(),
            1.0,
        );
        let result = ui.with_clip(viewport, || self.draw_map_content(rect, data, textures));
        self.draw_map_controls(rect);
        result
    }

    fn draw_map_content(
        &mut self,
        rect: Rect,
        data: &GameData,
        textures: &TextureManager,
    ) -> Option<UpdateResult> {
        self.draw_mountain_vista(rect);
        let map_rect = self.sect_map_view_rect(rect);
        draw_rectangle(
            map_rect.x,
            map_rect.y,
            map_rect.w,
            map_rect.h,
            Color::new(0.04, 0.035, 0.025, 0.20),
        );
        draw_rectangle_lines(
            map_rect.x,
            map_rect.y,
            map_rect.w,
            map_rect.h,
            1.5,
            Color::new(PRIMARY.r, PRIMARY.g, PRIMARY.b, 0.24),
        );
        let mouse = vec2(mouse_position().0, mouse_position().1);

        if map_rect.contains(mouse.into()) {
            let wheel = mouse_wheel().1;
            if wheel.abs() > 0.0 {
                let prev_zoom = self.map_zoom;
                self.map_zoom = (self.map_zoom * (1.0 + wheel * 0.1)).clamp(0.2, 2.0);
                let world_before = self.screen_to_world_with_zoom(map_rect, mouse, prev_zoom);
                let world_after = self.screen_to_world(map_rect, mouse);
                self.map_center += world_before - world_after;
            }
        }

        self.handle_map_drag(map_rect, mouse);
        self.clamp_map_center(map_rect);
        self.draw_map_base(map_rect, textures);
        self.draw_spirit_terraces(map_rect);
        self.draw_placement_preview(map_rect);

        if let Some(res) = self.handle_placement_click(map_rect, textures, data, mouse) {
            return Some(res);
        }

        let hovered_building_id =
            self.draw_buildings_and_detect_hover(map_rect, data, textures, mouse);

        if let Some(id) = hovered_building_id {
            if is_mouse_button_pressed(MouseButton::Left) {
                self.view = SectView::BuildingDetails(id);
            }
        }

        None
    }

    fn draw_map_controls(&mut self, rect: Rect) -> bool {
        let y = rect.y + rect.h - 50.0;
        let controls = ["+", "-", "Left", "Right", "Up", "Down", "Home"];
        let width = ((rect.w - 120.0) / 7.0).min(70.0);
        for (index, label) in controls.iter().enumerate() {
            if draw_button(
                Rect::new(rect.x + index as f32 * (width + 4.0), y, width, 44.0),
                label,
                false,
            ) {
                match index {
                    0 => self.map_zoom = (self.map_zoom * 1.2).min(2.0),
                    1 => self.map_zoom = (self.map_zoom / 1.2).max(0.2),
                    2 => self.map_center.x -= 120.0 / self.map_zoom,
                    3 => self.map_center.x += 120.0 / self.map_zoom,
                    4 => self.map_center.y -= 120.0 / self.map_zoom,
                    5 => self.map_center.y += 120.0 / self.map_zoom,
                    _ => {
                        self.map_center = vec2(SECT_MAP_WIDTH / 2.0, SECT_MAP_HEIGHT / 2.0);
                        self.map_zoom = 0.35;
                    }
                }
            }
        }
        if self.placement_mode.is_some()
            && draw_button(
                Rect::new(rect.x + rect.w - 90.0, y, 90.0, 44.0),
                "Cancel",
                false,
            )
        {
            self.placement_mode = None;
            return true;
        }
        false
    }

    fn sect_map_view_rect(&self, rect: Rect) -> Rect {
        Rect::new(rect.x, rect.y, rect.w, rect.h - 60.0)
    }

    fn handle_map_drag(&mut self, rect: Rect, mouse: Vec2) {
        let drag_active = is_mouse_button_down(MouseButton::Middle)
            || (is_key_down(KeyCode::Space) && is_mouse_button_down(MouseButton::Left));

        if drag_active && rect.contains(mouse.into()) {
            if !self.map_dragging {
                self.map_dragging = true;
                self.map_last_mouse = mouse;
            } else {
                let delta = mouse - self.map_last_mouse;
                self.map_center -= delta / self.map_zoom;
                self.map_last_mouse = mouse;
            }
        } else {
            self.map_dragging = false;
        }
    }

    fn draw_map_base(&self, rect: Rect, textures: &TextureManager) {
        if let Some(tex) = textures.get("sect_map_base") {
            let top_left = self.world_to_screen(rect, vec2(0.0, 0.0));
            draw_texture_ex(
                tex,
                top_left.x,
                top_left.y,
                Color::new(1.0, 0.96, 0.84, 0.62),
                DrawTextureParams {
                    dest_size: Some(vec2(
                        SECT_MAP_WIDTH * self.map_zoom,
                        SECT_MAP_HEIGHT * self.map_zoom,
                    )),
                    ..Default::default()
                },
            );
        } else {
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(0.12, 0.11, 0.10, 1.0),
            );
        }
    }

    pub(super) fn world_to_screen(&self, rect: Rect, world: Vec2) -> Vec2 {
        let view_center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        view_center + (world - self.map_center) * self.map_zoom
    }

    pub(super) fn screen_to_world(&self, rect: Rect, screen: Vec2) -> Vec2 {
        self.screen_to_world_with_zoom(rect, screen, self.map_zoom)
    }

    pub(super) fn screen_to_world_with_zoom(&self, rect: Rect, screen: Vec2, zoom: f32) -> Vec2 {
        let view_center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        self.map_center + (screen - view_center) / zoom
    }

    pub(super) fn clamp_map_center(&mut self, rect: Rect) {
        let half_view = vec2(
            rect.w / (2.0 * self.map_zoom),
            rect.h / (2.0 * self.map_zoom),
        );
        let min_x = half_view.x;
        let min_y = half_view.y;
        let max_x = SECT_MAP_WIDTH - half_view.x;
        let max_y = SECT_MAP_HEIGHT - half_view.y;

        if max_x < min_x {
            self.map_center.x = SECT_MAP_WIDTH * 0.5;
        } else {
            self.map_center.x = self.map_center.x.clamp(min_x, max_x);
        }

        if max_y < min_y {
            self.map_center.y = SECT_MAP_HEIGHT * 0.5;
        } else {
            self.map_center.y = self.map_center.y.clamp(min_y, max_y);
        }
    }
}
