//! Sect navigation, treasury and on-demand history.
use super::*;
use macroquad_toolkit::ui::draw_ui_text;

impl SectBaseState {
    pub(super) fn draw_header(
        &mut self,
        screen_w: f32,
        header_h: f32,
        spirit_stones: u32,
        herbs: u32,
        influence: u32,
        relics: u32,
        sect_name: &str,
        current_season: &Season,
        season_ticks: u32,
        tutorial: &mut TutorialState,
    ) {
        draw_panel(Rect::new(0.0, 0.0, screen_w, header_h), None);
        let title = if sect_name == "Unnamed Sect" {
            "Fallen Peak Sect"
        } else {
            sect_name
        };
        draw_ui_text(title, 24.0, 38.0, FONT_HEADER_SIZE, PRIMARY);
        herbs::draw_season_indicator(24.0, 65.0, current_season, season_ticks);
        let resources = [
            ("Stones", spirit_stones, PRIMARY),
            ("Herbs", herbs, SUCCESS),
            ("Prestige", influence, SECONDARY),
            ("Relics", relics, WARNING),
        ];
        let start = (screen_w * 0.30).max(240.0);
        let cell_w = (screen_w - start - 24.0) / 4.0;
        for (i, (label, value, color)) in resources.iter().enumerate() {
            let x = start + i as f32 * cell_w;
            draw_ui_text(label, x, 30.0, FONT_SMALL_SIZE, TEXT_SECONDARY);
            draw_ui_text(&value.to_string(), x, 58.0, FONT_HEADER_SIZE, *color);
        }
        if draw_button(
            Rect::new(screen_w - 118.0, 66.0, 102.0, 40.0),
            "Settings",
            false,
        ) {
            self.settings_open = true;
        }
        if draw_button(
            Rect::new(screen_w - 230.0, 66.0, 102.0, 40.0),
            "Annals",
            false,
        ) {
            self.annals_open = true;
            self.annals_page = 0;
        }
        if tutorial.active
            && tutorial.hidden
            && draw_button(
                Rect::new(screen_w - 342.0, 66.0, 102.0, 40.0),
                "Guide",
                false,
            )
        {
            tutorial.hidden = false;
        }
    }

    pub(super) fn draw_left_panel(
        &mut self,
        top: f32,
        screen_h: f32,
        width: f32,
        data: &GameData,
    ) -> Option<UpdateResult> {
        let rect = Rect::new(16.0, top + 12.0, width, screen_h - top - 24.0);
        draw_panel(rect, Some("Sect Grounds"));
        let x = rect.x + 10.0;
        if draw_button(
            Rect::new(x, rect.y + 50.0, width - 20.0, 44.0),
            "Mountain Map",
            matches!(self.view, SectView::Map),
        ) {
            self.view = SectView::Map;
        }
        let nav_top = rect.y + rect.h - 174.0;
        let per_page = ((nav_top - rect.y - 150.0) / 46.0).floor().max(1.0) as usize;
        let pages = data.buildings.len().div_ceil(per_page).max(1);
        self.building_page = self.building_page.min(pages - 1);
        for (row, building) in data
            .buildings
            .iter()
            .skip(self.building_page * per_page)
            .take(per_page)
            .enumerate()
        {
            let status = match building.status {
                BuildingStatus::Active => "",
                BuildingStatus::Ruined => " / Ruin",
                BuildingStatus::Constructing => " / Raising",
            };
            let label = format!("{}{}", building.building_type, status);
            let btn = Rect::new(x, rect.y + 102.0 + row as f32 * 46.0, width - 20.0, 44.0);
            match building.building_type {
                BuildingType::SectHall => {
                    for step in [0, 1, 3] {
                        self.register_tutorial_target(step, btn);
                    }
                }
                BuildingType::MissionBoard => self.register_tutorial_target(4, btn),
                _ => {}
            }
            if draw_button(
                btn,
                &label,
                self.view == SectView::BuildingDetails(building.id),
            ) {
                self.view = SectView::BuildingDetails(building.id);
            }
        }
        if pages > 1 {
            let y = nav_top - 50.0;
            if self.building_page > 0 && draw_button(Rect::new(x, y, 64.0, 44.0), "Prev", false) {
                self.building_page -= 1;
            }
            draw_ui_text(
                &format!("{}/{}", self.building_page + 1, pages),
                x + 82.0,
                y + 28.0,
                FONT_SMALL_SIZE,
                TEXT_SECONDARY,
            );
            if self.building_page + 1 < pages
                && draw_button(Rect::new(x + width - 84.0, y, 64.0, 44.0), "Next", false)
            {
                self.building_page += 1;
            }
        }
        draw_ink_divider(x, nav_top - 8.0, width - 20.0);
        let labels = [
            "Disciples",
            "Routes",
            "Powers",
            "Trade",
            "Raise Halls",
            "Guardians",
        ];
        let cell = (width - 24.0) / 2.0;
        for (index, label) in labels.iter().enumerate() {
            let btn = Rect::new(
                x + (index % 2) as f32 * (cell + 4.0),
                nav_top + (index / 2) as f32 * 54.0,
                cell,
                48.0,
            );
            if index == 4 {
                self.register_tutorial_target(2, btn);
            }
            if draw_button(btn, label, false) {
                let transition = match index {
                    0 => Some(StateTransition::ToDiscipleRoster),
                    1 => Some(StateTransition::ToWorldMap),
                    2 => Some(StateTransition::ToFactionScreen),
                    3 => Some(StateTransition::ToTradeScreen),
                    4 => {
                        self.view = SectView::Map;
                        self.crafting_modal_open = true;
                        None
                    }
                    _ => {
                        self.view = SectView::SpiritBeasts;
                        None
                    }
                };
                if let Some(transition) = transition {
                    return Some(UpdateResult::new().with_transition(transition));
                }
            }
        }
        None
    }

    pub(super) fn draw_annals(&mut self, events: &[String]) {
        let rect = Rect::new(24.0, 24.0, screen_width() - 48.0, screen_height() - 48.0);
        draw_panel(rect, Some("Sect Annals"));
        if draw_button(
            Rect::new(rect.x + rect.w - 132.0, rect.y + 8.0, 116.0, 44.0),
            "Back to Sect",
            false,
        ) {
            self.annals_open = false;
        }
        draw_ui_text(
            "Latest events first",
            rect.x + 24.0,
            rect.y + 76.0,
            FONT_SMALL_SIZE,
            SECONDARY,
        );
        let mut pages: Vec<Vec<&String>> = vec![Vec::new()];
        let available = rect.h - 180.0;
        let mut used = 0.0;
        for event in events.iter().rev() {
            let lines = macroquad_toolkit::ui::wrap_text(event, rect.w - 64.0, FONT_BODY_SIZE);
            let needed = lines.len() as f32 * (FONT_BODY_SIZE + 5.0) + 16.0;
            if used + needed > available && !pages.last().unwrap().is_empty() {
                pages.push(Vec::new());
                used = 0.0;
            }
            pages.last_mut().unwrap().push(event);
            used += needed;
        }
        self.annals_page = self.annals_page.min(pages.len() - 1);
        let mut y = rect.y + 112.0;
        let ui = macroquad_toolkit::ui::VirtualUi::from_responsive_screen_size(
            screen_width(),
            screen_height(),
            1.0,
        );
        ui.with_clip(
            Rect::new(rect.x, y - 24.0, rect.w, available + 24.0),
            || {
                for event in &pages[self.annals_page] {
                    y = draw_wrapped_text(
                        event,
                        rect.x + 24.0,
                        y,
                        rect.w - 64.0,
                        FONT_BODY_SIZE,
                        TEXT_PRIMARY,
                    ) + 16.0;
                }
            },
        );
        let footer = rect.y + rect.h - 56.0;
        if self.annals_page > 0
            && draw_button(
                Rect::new(rect.x + 24.0, footer, 120.0, 44.0),
                "Newer",
                false,
            )
        {
            self.annals_page -= 1;
        }
        if self.annals_page + 1 < pages.len()
            && draw_button(
                Rect::new(rect.x + rect.w - 144.0, footer, 120.0, 44.0),
                "Older",
                false,
            )
        {
            self.annals_page += 1;
        }
        draw_ui_text(
            &format!("{} / {}", self.annals_page + 1, pages.len()),
            rect.x + rect.w * 0.5,
            footer + 28.0,
            FONT_SMALL_SIZE,
            TEXT_SECONDARY,
        );
        if events.is_empty() {
            draw_ui_text(
                "Your sect's story begins here.",
                rect.x + 24.0,
                y,
                FONT_BODY_SIZE,
                TEXT_SECONDARY,
            );
        }
    }

    pub(super) fn draw_settings_modal(
        &mut self,
        screen_w: f32,
        screen_h: f32,
    ) -> Option<UpdateResult> {
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.8));

        let modal_w = 300.0;
        let modal_h = 250.0;
        let modal_x = (screen_w - modal_w) / 2.0;
        let modal_y = (screen_h - modal_h) / 2.0;
        let modal_rect = Rect::new(modal_x, modal_y, modal_w, modal_h);

        draw_panel(modal_rect, Some("Patriarch's Seal"));

        if draw_button(
            Rect::new(modal_x + 50.0, modal_y + 60.0, 200.0, 40.0),
            "Inscribe Chronicle",
            false,
        ) {
            return Some(UpdateResult::new().with_action(Action::SaveGame));
        }

        if draw_button(
            Rect::new(modal_x + 50.0, modal_y + 120.0, 200.0, 40.0),
            "Return to Title",
            false,
        ) {
            return Some(UpdateResult::new().with_transition(StateTransition::ToMainMenu));
        }

        if draw_button(
            Rect::new(modal_x + 50.0, modal_y + 180.0, 200.0, 40.0),
            "Close Seal",
            false,
        ) {
            self.settings_open = false;
        }
        None
    }
}
