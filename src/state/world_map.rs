use crate::data::factions::{Faction, FactionType};
use crate::data::loader::GameData;
use crate::data::missions::MapNode;
use crate::state::{StateTransition, UpdateResult};
use crate::ui::components::*;
use crate::ui::theme::*;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub struct WorldMapState {
    selected: usize,
    page: usize,
}

impl WorldMapState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            page: 0,
        }
    }

    pub fn update(&mut self, data: &GameData) -> UpdateResult {
        use crate::ui::workspace as ux;
        if ux::header(
            "Outside Routes",
            "Select a region to inspect danger. Issue expeditions from the Mission Board.",
        ) {
            return UpdateResult::new().with_transition(StateTransition::ToSectBase);
        }
        let (list, detail) = ux::split(ux::content());
        self.selected = self.selected.min(data.map_nodes.len().saturating_sub(1));
        let range = ux::page(list, data.map_nodes.len(), 64.0, &mut self.page);
        for (row, index) in range.enumerate() {
            if ux::button(
                Rect::new(list.x, list.y + row as f32 * 64.0, list.w, 52.0),
                &data.map_nodes[index].name,
                self.selected == index,
                true,
            ) {
                self.selected = index;
            }
        }
        let map = Rect::new(detail.x, detail.y, detail.w, detail.h - 112.0);
        ux::clipped(map, || {
            self.draw_map_ground(map);
            let home = vec2(map.x + map.w * 0.47, map.y + map.h * 0.60);
            for (index, node) in data.map_nodes.iter().enumerate() {
                let position = map_node_pos(map, node);
                let selected = index == self.selected;
                if selected {
                    draw_route(home, position, PRIMARY);
                }
                draw_circle(
                    position.x,
                    position.y,
                    if selected { 10.0 } else { 5.0 },
                    if selected { PRIMARY } else { PANEL_BORDER },
                );
                if selected {
                    draw_circle_lines(position.x, position.y, 16.0, 2.0, PRIMARY);
                }
            }
            self.draw_player_sect(home);
        });
        let info = Rect::new(detail.x, detail.y + detail.h - 100.0, detail.w, 100.0);
        draw_panel(info, None);
        if let Some(node) = data.map_nodes.get(self.selected) {
            draw_ui_text(
                &node.name,
                info.x + 16.0,
                info.y + 30.0,
                FONT_BODY_SIZE,
                PRIMARY,
            );
            draw_ui_text(
                &format!(
                    "Threat {} / Corruption {}",
                    node.danger_level + node.corruption / 10,
                    node.corruption
                ),
                info.x + 16.0,
                info.y + 62.0,
                FONT_BODY_SIZE,
                TEXT_PRIMARY,
            );
        }
        UpdateResult::new()
    }

    fn draw_map_ground(&self, rect: Rect) {
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.66, 0.56, 0.39, 0.92),
        );
        draw_rectangle(
            rect.x + 8.0,
            rect.y + 8.0,
            rect.w - 16.0,
            rect.h - 16.0,
            Color::new(0.78, 0.69, 0.49, 0.70),
        );
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, PANEL_BORDER);
        draw_rectangle_lines(
            rect.x + 8.0,
            rect.y + 8.0,
            rect.w - 16.0,
            rect.h - 16.0,
            1.0,
            Color::new(PRIMARY.r, PRIMARY.g, PRIMARY.b, 0.42),
        );

        self.draw_coastline(rect);
        self.draw_rivers(rect);
        self.draw_mountain_glyphs(rect);
        self.draw_fog_bank(rect);

        for i in 0..7 {
            let y = rect.y + rect.h * (0.18 + i as f32 * 0.09);
            draw_line(
                rect.x + rect.w * 0.12,
                y,
                rect.x + rect.w * 0.88,
                y + (i as f32 * 7.0).sin() * 10.0,
                1.0,
                Color::new(TEXT_SECONDARY.r, TEXT_SECONDARY.g, TEXT_SECONDARY.b, 0.08),
            );
        }

        draw_ui_text(
            "Outer Realm Ink Map",
            rect.x + 22.0,
            rect.y + 34.0,
            FONT_HEADER_SIZE,
            Color::new(0.20, 0.12, 0.07, 0.74),
        );
    }

    fn draw_coastline(&self, rect: Rect) {
        let coast_x = rect.x + rect.w * 0.83;
        draw_rectangle(
            coast_x,
            rect.y + 10.0,
            rect.w * 0.15,
            rect.h - 20.0,
            Color::new(0.14, 0.42, 0.50, 0.22),
        );
        for i in 0..12 {
            let y = rect.y + 24.0 + i as f32 * rect.h / 12.0;
            draw_line(
                coast_x + (i as f32 * 1.7).sin() * 12.0,
                y,
                coast_x + 36.0 + (i as f32 * 2.3).cos() * 18.0,
                y + 18.0,
                2.0,
                Color::new(0.08, 0.22, 0.24, 0.18),
            );
        }
    }

    fn draw_rivers(&self, rect: Rect) {
        let rivers = [
            [
                (0.18, 0.18),
                (0.28, 0.32),
                (0.38, 0.45),
                (0.58, 0.52),
                (0.82, 0.58),
            ],
            [
                (0.10, 0.72),
                (0.26, 0.64),
                (0.42, 0.68),
                (0.60, 0.76),
                (0.84, 0.70),
            ],
        ];
        for river in rivers {
            for pair in river.windows(2) {
                let a = vec2(rect.x + rect.w * pair[0].0, rect.y + rect.h * pair[0].1);
                let b = vec2(rect.x + rect.w * pair[1].0, rect.y + rect.h * pair[1].1);
                draw_line(a.x, a.y, b.x, b.y, 9.0, Color::new(0.28, 0.60, 0.64, 0.18));
                draw_line(a.x, a.y, b.x, b.y, 3.0, Color::new(0.13, 0.30, 0.34, 0.28));
            }
        }
    }

    fn draw_mountain_glyphs(&self, rect: Rect) {
        let clusters = [
            (0.16, 0.28, 5),
            (0.34, 0.24, 4),
            (0.48, 0.36, 6),
            (0.24, 0.56, 4),
            (0.66, 0.42, 5),
            (0.58, 0.68, 5),
            (0.74, 0.24, 3),
        ];
        for (x, y, count) in clusters {
            for i in 0..count {
                let px = rect.x + rect.w * x + i as f32 * 14.0;
                let py = rect.y + rect.h * y + ((i % 2) as f32 * 12.0);
                draw_triangle(
                    vec2(px - 13.0, py + 16.0),
                    vec2(px, py - 12.0),
                    vec2(px + 15.0, py + 16.0),
                    Color::new(0.20, 0.14, 0.08, 0.26),
                );
                draw_line(
                    px,
                    py - 10.0,
                    px - 5.0,
                    py + 12.0,
                    1.0,
                    Color::new(0.88, 0.80, 0.62, 0.18),
                );
            }
        }
    }

    fn draw_fog_bank(&self, rect: Rect) {
        let fog = [
            (0.12, 0.14, 38.0),
            (0.30, 0.82, 46.0),
            (0.52, 0.18, 34.0),
            (0.74, 0.78, 54.0),
            (0.88, 0.34, 44.0),
        ];
        for (x, y, r) in fog {
            draw_circle(
                rect.x + rect.w * x,
                rect.y + rect.h * y,
                r,
                Color::new(0.88, 0.84, 0.72, 0.16),
            );
            draw_circle(
                rect.x + rect.w * x + r * 0.72,
                rect.y + rect.h * y + 8.0,
                r * 0.74,
                Color::new(0.88, 0.84, 0.72, 0.10),
            );
        }
    }

    fn draw_player_sect(&self, pos: Vec2) {
        draw_circle(
            pos.x,
            pos.y,
            30.0,
            Color::new(PRIMARY.r, PRIMARY.g, PRIMARY.b, 0.26),
        );
        draw_circle_lines(pos.x, pos.y, 36.0, 2.0, PRIMARY);
        draw_circle(pos.x, pos.y, 10.0, PRIMARY);
        draw_ui_text(
            "Your Sect",
            pos.x - 44.0,
            pos.y + 58.0,
            FONT_HEADER_SIZE,
            TEXT_HIGHLIGHT,
        );
        draw_ui_text(
            "Fallen peak",
            pos.x - 30.0,
            pos.y + 78.0,
            FONT_SMALL_SIZE,
            TEXT_SECONDARY,
        );
    }

    pub fn draw(&self, _data: &GameData, _spirit_stones: u32) {
        // Handled in update.
    }
}

fn map_node_pos(rect: Rect, node: &MapNode) -> Vec2 {
    let x = (node.x / 500.0).clamp(0.10, 0.90);
    let y = (node.y / 500.0).clamp(0.12, 0.86);
    vec2(rect.x + rect.w * x, rect.y + rect.h * y)
}

fn draw_route(a: Vec2, b: Vec2, color: Color) {
    draw_line(
        a.x,
        a.y,
        b.x,
        b.y,
        5.0,
        Color::new(color.r, color.g, color.b, color.a * 0.28),
    );
    draw_line(a.x, a.y, b.x, b.y, 2.0, color);
    let mid = (a + b) * 0.5;
    draw_circle(
        mid.x,
        mid.y,
        2.5,
        Color::new(color.r, color.g, color.b, color.a + 0.12),
    );
}
