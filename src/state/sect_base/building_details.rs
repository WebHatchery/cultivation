use super::*;
use macroquad_toolkit::ui::draw_ui_text;

impl SectBaseState {
    pub(super) fn draw_building_details(
        &mut self,
        rect: Rect,
        id: u64,
        data: &mut GameData,
        spirit_stones: u32,
        herbs: u32,
        inventory: &std::collections::HashMap<String, u32>,
        unlocked_techs: &[String],
        ongoing_missions: &[OngoingMission],
        completed_missions: &[MissionOutcome],
        completed_history: &[String],
        disciples: &[Disciple],
        current_season: &Season,
        discovered_recipes: &[String],
    ) -> Option<UpdateResult> {
        let building = data.buildings.iter().find(|b| b.id == id)?.clone();
        let b_type = building.building_type.clone();

        if self.crafting_modal_open {
            return self.draw_building_modals(
                data,
                &b_type,
                spirit_stones,
                herbs,
                inventory,
                unlocked_techs,
                disciples,
                discovered_recipes,
            );
        }
        draw_panel(rect, None);
        self.draw_building_header(rect, &building, data, disciples);

        if draw_button(
            Rect::new(rect.x + rect.w - 80.0, rect.y + 10.0, 60.0, 30.0),
            "Map",
            false,
        ) {
            self.view = SectView::Map;
            self.herb_planting_modal = None;
            self.disciple_assignment_modal = false;
            self.infusion_modal_open = false;
            return None;
        }

        let action_y = self.calculate_action_y(rect, &building, data, disciples);

        if let Some(res) = self.dispatch_building_type(
            rect,
            &building,
            &b_type,
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
            action_y,
        ) {
            return Some(res);
        }

        self.draw_building_modals(
            data,
            &b_type,
            spirit_stones,
            herbs,
            inventory,
            unlocked_techs,
            disciples,
            discovered_recipes,
        )
    }

    fn draw_building_header(
        &self,
        rect: Rect,
        building: &crate::data::buildings::Building,
        data: &GameData,
        disciples: &[Disciple],
    ) {
        let b_type = &building.building_type;
        let status_label = match building.status {
            BuildingStatus::Active => "Active Hall",
            BuildingStatus::Ruined => "Ruined Hall",
            BuildingStatus::Constructing => "Being Raised",
        };
        draw_ui_text(
            &format!("{}", b_type),
            rect.x + 20.0,
            rect.y + 38.0,
            FONT_HEADER_SIZE,
            PRIMARY,
        );
        draw_ui_text(
            status_label,
            rect.x + 20.0,
            rect.y + 66.0,
            FONT_SMALL_SIZE,
            status_color(&building.status),
        );
        let d_y = rect.y + 82.0;
        draw_ui_text(
            &format!("Hall Grade: {}", building.level),
            rect.x + 20.0,
            d_y + 14.0,
            FONT_BODY_SIZE,
            TEXT_PRIMARY,
        );
        draw_ui_text(
            &format!("Aspect: {}", building.element),
            rect.x + 20.0,
            d_y + 44.0,
            FONT_BODY_SIZE,
            TEXT_SECONDARY,
        );
        let fs_color = if building.feng_shui_score > 0.0 {
            Color::new(0.2, 0.8, 0.2, 1.0)
        } else if building.feng_shui_score < 0.0 {
            Color::new(0.8, 0.2, 0.2, 1.0)
        } else {
            TEXT_SECONDARY
        };
        draw_ui_text(
            &format!("Feng Shui: {:.1}", building.feng_shui_score),
            rect.x + 20.0,
            d_y + 74.0,
            FONT_BODY_SIZE,
            fs_color,
        );

        if let Some(def) = data.building_definitions.get(&building.building_type) {
            draw_wrapped_text(
                &def.description,
                rect.x + rect.w * 0.48,
                rect.y + 100.0,
                rect.w * 0.52 - 24.0,
                FONT_BODY_SIZE,
                TEXT_SECONDARY,
            );
        }
    }

    fn calculate_action_y(
        &self,
        rect: Rect,
        building: &crate::data::buildings::Building,
        data: &GameData,
        disciples: &[Disciple],
    ) -> f32 {
        let d_y = rect.y + 82.0;
        let mut action_y = d_y + 100.0;

        if building.building_type == BuildingType::SectHall {
            let capacity = self.calculate_population_capacity_from_buildings(&data.buildings);
            draw_ui_text(
                &format!("Sect census: {}/{}", disciples.len(), capacity),
                rect.x + 20.0,
                d_y + 104.0,
                FONT_BODY_SIZE,
                TEXT_SECONDARY,
            );
            action_y = d_y + 130.0;
        }

        action_y
    }

    fn calculate_population_capacity_from_buildings(
        &self,
        buildings: &[crate::data::buildings::Building],
    ) -> u32 {
        buildings
            .iter()
            .filter(|b| b.status == BuildingStatus::Active)
            .map(|b| b.get_max_disciples() + b.get_dorm_capacity())
            .sum()
    }

    fn dispatch_building_type(
        &mut self,
        rect: Rect,
        building: &crate::data::buildings::Building,
        b_type: &BuildingType,
        data: &mut GameData,
        spirit_stones: u32,
        herbs: u32,
        inventory: &std::collections::HashMap<String, u32>,
        unlocked_techs: &[String],
        ongoing_missions: &[OngoingMission],
        completed_missions: &[MissionOutcome],
        completed_history: &[String],
        disciples: &[Disciple],
        current_season: &Season,
        discovered_recipes: &[String],
        action_y: f32,
    ) -> Option<UpdateResult> {
        if building.status == BuildingStatus::Ruined {
            return self.draw_ruined_building(rect, building, b_type, action_y);
        }

        if *b_type == BuildingType::MissionBoard {
            return self.draw_mission_list(
                rect,
                data,
                ongoing_missions,
                completed_missions,
                completed_history,
                action_y,
            );
        }

        if matches!(b_type, BuildingType::HerbGarden | BuildingType::Greenhouse) {
            return self.draw_herb_building_details(
                rect,
                building,
                data,
                disciples,
                current_season,
                action_y,
            );
        }

        if *b_type == BuildingType::DryingPavilion {
            return self.draw_drying_pavilion_details(rect, building, data, inventory, action_y);
        }

        if *b_type == BuildingType::HerbStorage {
            return self.draw_herb_storage_details(rect, building, data, inventory, action_y);
        }

        self.draw_generic_building_actions(rect, b_type, action_y)
    }

    fn draw_ruined_building(
        &mut self,
        rect: Rect,
        building: &crate::data::buildings::Building,
        b_type: &BuildingType,
        action_y: f32,
    ) -> Option<UpdateResult> {
        draw_ui_text(
            "(Ruined)",
            rect.x + 200.0,
            rect.y + 38.0,
            FONT_HEADER_SIZE,
            Color::new(0.8, 0.2, 0.2, 1.0),
        );
        draw_wrapped_text(
            "Broken beams and cold incense mark a debt the patriarch can repay.",
            rect.x + 20.0,
            action_y - 42.0,
            rect.w - 40.0,
            FONT_BODY_SIZE,
            TEXT_SECONDARY,
        );
        let repair_cost = building.repair_cost;
        let repair_label = if *b_type == BuildingType::SectHall {
            format!("Restore Hall ({} SS)", repair_cost)
        } else {
            format!("Raise Beams ({} SS)", repair_cost)
        };
        let repair_rect = Rect::new(rect.x + 20.0, action_y, 200.0, 40.0);
        if *b_type == BuildingType::SectHall {
            self.register_tutorial_target(0, repair_rect);
        }
        if draw_button(repair_rect, &repair_label, false) {
            return Some(UpdateResult::new().with_action(Action::RepairBuilding(building.id)));
        }
        None
    }

    fn draw_generic_building_actions(
        &mut self,
        rect: Rect,
        b_type: &BuildingType,
        action_y: f32,
    ) -> Option<UpdateResult> {
        let x = rect.x + 20.0;
        let button_w = (rect.w - 56.0) / 3.0;
        if *b_type == BuildingType::SectHall {
            let doctrine = Rect::new(x, action_y, button_w, 48.0);
            self.register_tutorial_target(1, doctrine);
            if draw_button(doctrine, "Recover Doctrine", true) {
                self.tech_tree_open = true;
            }
            let recruit = Rect::new(x + button_w + 8.0, action_y, button_w, 48.0);
            self.register_tutorial_target(3, recruit);
            if draw_button(recruit, "Accept Disciple", false) {
                return Some(UpdateResult::new().with_action(Action::RecruitDisciple));
            }
            if draw_button(
                Rect::new(x + 2.0 * (button_w + 8.0), action_y, button_w, 48.0),
                "Raise Grade (50 SS)",
                false,
            ) {
                return Some(
                    UpdateResult::new().with_action(Action::UpgradeBuilding(b_type.clone())),
                );
            }
        } else {
            if draw_button(
                Rect::new(x, action_y, button_w, 48.0),
                "Raise Grade (50 SS)",
                false,
            ) {
                return Some(
                    UpdateResult::new().with_action(Action::UpgradeBuilding(b_type.clone())),
                );
            }
            if matches!(
                b_type,
                BuildingType::AlchemyFurnace
                    | BuildingType::ArtifactForge
                    | BuildingType::Blacksmith
                    | BuildingType::TalismanScriptorium
            ) && draw_button(
                Rect::new(x + button_w + 8.0, action_y, button_w, 48.0),
                "Open Workshop",
                false,
            ) {
                self.crafting_modal_open = true;
            }
        }
        None
    }

    fn draw_building_modals(
        &mut self,
        data: &GameData,
        b_type: &BuildingType,
        spirit_stones: u32,
        herbs: u32,
        inventory: &std::collections::HashMap<String, u32>,
        unlocked_techs: &[String],
        disciples: &[Disciple],
        discovered_recipes: &[String],
    ) -> Option<UpdateResult> {
        if self.crafting_modal_open {
            if let Some(res) = self.draw_crafting_modal(
                data,
                b_type,
                spirit_stones,
                herbs,
                inventory,
                disciples,
                discovered_recipes,
            ) {
                return Some(res);
            }
        }

        None
    }

    fn draw_herb_building_details(
        &mut self,
        rect: Rect,
        building: &crate::data::buildings::Building,
        data: &GameData,
        disciples: &[Disciple],
        current_season: &Season,
        action_y: f32,
    ) -> Option<UpdateResult> {
        let b_type = &building.building_type;

        if draw_button(
            Rect::new(rect.x + 20.0, action_y, 140.0, 40.0),
            "Raise Hall (50 SS)",
            false,
        ) {
            return Some(UpdateResult::new().with_action(Action::UpgradeBuilding(b_type.clone())));
        }

        if draw_button(
            Rect::new(rect.x + 170.0, action_y, 140.0, 40.0),
            "Appoint",
            false,
        ) {
            self.disciple_assignment_modal = true;
        }

        if *b_type == BuildingType::Greenhouse {
            if draw_button(
                Rect::new(rect.x + 320.0, action_y, 140.0, 40.0),
                "Tune Array",
                false,
            ) {
                self.infusion_modal_open = true;
            }
        }

        let panel_y = action_y + 60.0;
        if *b_type == BuildingType::Greenhouse {
            herbs::draw_greenhouse_panel(
                building,
                data,
                disciples,
                current_season,
                rect.x + 20.0,
                panel_y,
                rect.w - 40.0,
            );
        } else {
            herbs::draw_herb_garden_panel(
                building,
                data,
                disciples,
                current_season,
                rect.x + 20.0,
                panel_y,
                rect.w - 40.0,
            );
        }

        let mut plot_btn_y = panel_y + 30.0;
        for (i, plot) in building.herb_plots.iter().enumerate() {
            if i >= building.get_max_herb_plots() {
                break;
            }
            if plot.growing.is_none() {
                if draw_button(
                    Rect::new(rect.x + rect.w - 120.0, plot_btn_y + 10.0, 80.0, 30.0),
                    "Sow",
                    false,
                ) {
                    self.herb_planting_modal = Some(i);
                }
            }
            plot_btn_y += 65.0;
        }

        self.draw_herb_modals(building, data, disciples, current_season)
    }

    fn draw_herb_modals(
        &mut self,
        building: &crate::data::buildings::Building,
        data: &GameData,
        disciples: &[Disciple],
        current_season: &Season,
    ) -> Option<UpdateResult> {
        if let Some(plot_idx) = self.herb_planting_modal {
            let result = herbs::draw_herb_planting_modal(building, data, current_season, plot_idx);
            if result.close_modal {
                self.herb_planting_modal = None;
            }
            if let Some(action) = result.action {
                return Some(UpdateResult::new().with_action(action));
            }
        }

        if self.disciple_assignment_modal {
            let result =
                herbs::draw_disciple_assignment_modal(building, disciples, &data.buildings);
            if result.close_modal {
                self.disciple_assignment_modal = false;
            }
            if let Some(action) = result.action {
                return Some(UpdateResult::new().with_action(action));
            }
        }

        if self.infusion_modal_open {
            let result = herbs::draw_infusion_modal(building);
            if result.close_modal {
                self.infusion_modal_open = false;
            }
            if let Some(action) = result.action {
                return Some(UpdateResult::new().with_action(action));
            }
        }

        None
    }

    fn draw_drying_pavilion_details(
        &mut self,
        rect: Rect,
        building: &crate::data::buildings::Building,
        data: &GameData,
        inventory: &std::collections::HashMap<String, u32>,
        action_y: f32,
    ) -> Option<UpdateResult> {
        if draw_button(
            Rect::new(rect.x + 20.0, action_y, 150.0, 40.0),
            "Raise Hall (50 SS)",
            false,
        ) {
            return Some(
                UpdateResult::new()
                    .with_action(Action::UpgradeBuilding(building.building_type.clone())),
            );
        }

        let panel_y = action_y + 60.0;
        if let Some(action) = herbs::draw_drying_pavilion_panel(
            building,
            data,
            inventory,
            rect.x + 20.0,
            panel_y,
            rect.w - 40.0,
        ) {
            return Some(UpdateResult::new().with_action(action));
        }

        None
    }

    fn draw_herb_storage_details(
        &mut self,
        rect: Rect,
        building: &crate::data::buildings::Building,
        data: &GameData,
        inventory: &std::collections::HashMap<String, u32>,
        action_y: f32,
    ) -> Option<UpdateResult> {
        if draw_button(
            Rect::new(rect.x + 20.0, action_y, 150.0, 40.0),
            "Raise Hall (50 SS)",
            false,
        ) {
            return Some(
                UpdateResult::new()
                    .with_action(Action::UpgradeBuilding(building.building_type.clone())),
            );
        }

        let panel_y = action_y + 60.0;
        herbs::draw_herb_storage_panel(
            building,
            data,
            inventory,
            rect.x + 20.0,
            panel_y,
            rect.w - 40.0,
        );

        None
    }
}

fn status_color(status: &BuildingStatus) -> Color {
    match status {
        BuildingStatus::Active => SUCCESS,
        BuildingStatus::Ruined => FAILURE,
        BuildingStatus::Constructing => WARNING,
    }
}
