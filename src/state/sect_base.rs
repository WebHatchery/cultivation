mod building_details;
mod building_missions;
mod crafting;
mod map_view;
mod panels;
mod spirit_beasts;
mod tutorial;

use crate::data::buildings::{BuildingStatus, BuildingType};
use crate::data::disciples::Disciple;
use crate::data::elements::Element;
use crate::data::grid::Grid;
use crate::data::herbs::Season;
use crate::data::loader::GameData;
use crate::data::missions::{MissionOutcome, OngoingMission};
use crate::data::spirit_beasts::SpiritBeast;
use crate::engine::actions::Action;
use crate::state::{StateTransition, TutorialState, UpdateResult};
use crate::ui::components::*;
use crate::ui::herbs;
use crate::ui::tech::{self, TechTreeState};
use crate::ui::theme::*;
use crate::ui::TextureManager;
use macroquad::prelude::*;
use std::collections::HashMap;

pub(super) const SECT_MAP_WIDTH: f32 = 4096.0;
pub(super) const SECT_MAP_HEIGHT: f32 = 4096.0;
pub(super) const BUILDING_SCALE: f32 = 0.25;
pub(super) const ALPHA_THRESHOLD: u8 = 10;

#[derive(Clone, PartialEq)]
pub(super) enum SectView {
    Map,
    BuildingDetails(u64),
    SpiritBeasts,
}

pub struct SectBaseState {
    pub(super) view: SectView,
    pub(super) settings_open: bool,
    pub(super) annals_open: bool,
    pub(super) annals_page: usize,
    pub(super) building_page: usize,
    pub(super) feng_shui_overlay_active: bool,
    pub(super) crafting_modal_open: bool,
    pub(super) tech_tree_open: bool,
    pub(super) tech_tree_state: TechTreeState,
    pub(super) construction_scroll: f32,
    pub(super) placement_mode: Option<BuildingType>,
    pub(super) map_zoom: f32,
    pub(super) map_center: Vec2,
    pub(super) map_dragging: bool,
    pub(super) map_last_mouse: Vec2,
    pub(super) building_mask_cache: HashMap<String, Image>,
    pub(super) selected_beast_index: Option<usize>,
    pub(super) beast_equip_modal_open: bool,
    pub(super) selected_mission: Option<String>,
    pub(super) mission_scroll: f32,
    pub(super) herb_planting_modal: Option<usize>,
    pub(super) disciple_assignment_modal: bool,
    pub(super) infusion_modal_open: bool,
    pub(super) tutorial_step_active: Option<usize>,
    pub(super) tutorial_targets: Vec<Rect>,
}

impl SectBaseState {
    pub fn new() -> Self {
        Self {
            view: SectView::Map,
            settings_open: false,
            annals_open: false,
            annals_page: 0,
            building_page: 0,
            feng_shui_overlay_active: false,
            crafting_modal_open: false,
            tech_tree_open: false,
            tech_tree_state: TechTreeState::new(),
            construction_scroll: 0.0,
            placement_mode: None,
            map_zoom: 0.35,
            map_center: vec2(SECT_MAP_WIDTH / 2.0, SECT_MAP_HEIGHT / 2.0),
            map_dragging: false,
            map_last_mouse: vec2(0.0, 0.0),
            building_mask_cache: HashMap::new(),
            selected_beast_index: None,
            beast_equip_modal_open: false,
            selected_mission: None,
            mission_scroll: 0.0,
            herb_planting_modal: None,
            disciple_assignment_modal: false,
            infusion_modal_open: false,
            tutorial_step_active: None,
            tutorial_targets: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        data: &mut GameData,
        grid: &mut Grid,
        textures: &TextureManager,
        spirit_stones: u32,
        herbs: u32,
        influence: u32,
        relics: u32,
        sect_name: &str,
        inventory: &std::collections::HashMap<String, u32>,
        unlocked_techs: &[String],
        event_log: &[String],
        ongoing_missions: &[OngoingMission],
        completed_missions: &[MissionOutcome],
        completed_history: &[String],
        disciples: &[Disciple],
        spirit_beasts: &[SpiritBeast],
        current_season: &Season,
        season_ticks: u32,
        tutorial: &mut TutorialState,
        discovered_recipes: &[String],
    ) -> UpdateResult {
        if let Some(res) = self.handle_global_input() {
            return res;
        }

        let screen_w = screen_width();
        let screen_h = screen_height();
        let header_h = 112.0;
        let left_panel_w = 218.0;

        self.update_tutorial_progress(
            tutorial,
            data,
            unlocked_techs,
            disciples,
            ongoing_missions,
            completed_history,
        );
        self.tutorial_targets.clear();
        self.tutorial_step_active = if tutorial.active && !tutorial.hidden {
            Some(tutorial.step)
        } else {
            None
        };

        // Focused screens own input for the entire frame, including the close frame.
        if self.settings_open {
            return self
                .draw_settings_modal(screen_w, screen_h)
                .unwrap_or_else(UpdateResult::new);
        }
        if self.annals_open {
            self.draw_annals(event_log);
            if crate::ui::workspace::button(
                Rect::new(screen_w - 300.0, 32.0, 132.0, 44.0),
                "Open Archive",
                false,
                true,
            ) {
                return UpdateResult::new().with_transition(StateTransition::ToLibrary);
            }
            return UpdateResult::new();
        }
        if self.tech_tree_open {
            let action = tech::draw_tech_tree_modal(
                &mut self.tech_tree_state,
                data,
                unlocked_techs,
                spirit_stones,
            );
            if self.tech_tree_state.closed {
                self.tech_tree_open = false;
            }
            return action
                .map(|action| UpdateResult::new().with_action(action))
                .unwrap_or_else(UpdateResult::new);
        }
        if self.crafting_modal_open {
            let result = match self.view {
                SectView::Map => self.draw_construction_modal(data, unlocked_techs, spirit_stones),
                SectView::BuildingDetails(id) => {
                    if let Some(building) = data.buildings.iter().find(|b| b.id == id) {
                        self.draw_crafting_modal(
                            data,
                            &building.building_type,
                            spirit_stones,
                            herbs,
                            inventory,
                            disciples,
                            discovered_recipes,
                        )
                    } else {
                        self.crafting_modal_open = false;
                        None
                    }
                }
                SectView::SpiritBeasts => {
                    self.crafting_modal_open = false;
                    None
                }
            };
            return result.unwrap_or_else(UpdateResult::new);
        }
        let guidance_h = if tutorial.active && !tutorial.hidden {
            112.0
        } else {
            0.0
        };
        let content_top = header_h + guidance_h;
        let center_rect = Rect::new(
            left_panel_w + 32.0,
            content_top + 12.0,
            screen_w - left_panel_w - 48.0,
            screen_h - content_top - 24.0,
        );
        if let Some(res) = self.draw_center_panel(
            center_rect,
            data,
            grid,
            textures,
            spirit_stones,
            herbs,
            inventory,
            unlocked_techs,
            ongoing_missions,
            completed_missions,
            completed_history,
            disciples,
            spirit_beasts,
            current_season,
            discovered_recipes,
        ) {
            return res;
        }

        self.draw_header(
            screen_w,
            header_h,
            spirit_stones,
            herbs,
            influence,
            relics,
            sect_name,
            current_season,
            season_ticks,
            tutorial,
        );

        if let Some(res) = self.draw_left_panel(content_top, screen_h, left_panel_w, data) {
            return res;
        }

        self.draw_tutorial_highlights();
        if tutorial.active && !tutorial.hidden {
            self.draw_tutorial_overlay(screen_w, header_h, tutorial);
        }

        if self.settings_open {
            if let Some(res) = self.draw_settings_modal(screen_w, screen_h) {
                return res;
            }
        }

        UpdateResult::new()
    }

    /// Configure real UI states for the shared screenshot harness.
    pub fn prepare_capture(&mut self, scene: &str, hall_id: u64) {
        if scene == "hall" || scene == "doctrine" {
            self.view = SectView::BuildingDetails(hall_id);
        }
        self.tech_tree_open = scene == "doctrine";
        self.annals_open = scene == "annals";
        self.crafting_modal_open = scene == "construction";
    }

    pub fn draw(&self, _data: &GameData, _grid: &Grid, _spirit_stones: u32) {
        // Drawing handled in update for immediate mode
    }

    fn handle_global_input(&mut self) -> Option<UpdateResult> {
        if !is_key_pressed(KeyCode::Escape) {
            return None;
        }

        if self.settings_open {
            self.settings_open = false;
        } else if self.annals_open {
            self.annals_open = false;
        } else if self.tech_tree_open {
            self.tech_tree_open = false;
        } else if self.crafting_modal_open {
            self.crafting_modal_open = false;
        } else if self.placement_mode.is_some() {
            self.placement_mode = None;
        } else if let SectView::BuildingDetails(_) = self.view {
            self.view = SectView::Map;
        } else if let SectView::SpiritBeasts = self.view {
            self.view = SectView::Map;
        } else {
            return Some(UpdateResult::new().with_transition(StateTransition::ToMainMenu));
        }
        None
    }

    fn draw_center_panel(
        &mut self,
        rect: Rect,
        data: &mut GameData,
        grid: &mut Grid,
        textures: &TextureManager,
        spirit_stones: u32,
        herbs: u32,
        inventory: &std::collections::HashMap<String, u32>,
        unlocked_techs: &[String],
        ongoing_missions: &[OngoingMission],
        completed_missions: &[MissionOutcome],
        completed_history: &[String],
        disciples: &[Disciple],
        spirit_beasts: &[SpiritBeast],
        current_season: &Season,
        discovered_recipes: &[String],
    ) -> Option<UpdateResult> {
        match self.view {
            SectView::Map => self.draw_map_view(rect, data, textures),
            SectView::BuildingDetails(id) => self.draw_building_details(
                rect,
                id,
                data,
                spirit_stones,
                herbs,
                inventory,
                unlocked_techs,
                ongoing_missions,
                completed_missions,
                completed_history,
                disciples,
                current_season,
                discovered_recipes,
            ),
            SectView::SpiritBeasts => {
                self.draw_spirit_beasts_view(rect, data, spirit_beasts, inventory)
            }
        }
    }
}
