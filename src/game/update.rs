use super::Game;
use crate::data::buildings::{BuildingStatus, BuildingType};
use crate::data::disciples::DiscipleRank;
use crate::engine::world_sim::WorldSimResult;
use crate::state::{
    faction_screen::FactionScreenState, intro::IntroState, library::LibraryState,
    main_menu::MainMenuState, mission_assignment::MissionAssignmentState,
    mission_resolution::MissionResolutionState, roster::DiscipleRosterState,
    sect_base::SectBaseState, sect_creation::SectCreationState, trade_screen::TradeScreenState,
    tribulation::TribulationEncounterState, world_map::WorldMapState, GameState, StateTransition,
};
use crate::ui::components::{draw_panel, draw_progress_bar};
use crate::ui::theme::*;
use macroquad::prelude::{
    draw_rectangle, is_key_pressed, screen_height, screen_width, Color, KeyCode, Rect,
};
use macroquad_toolkit::rng as game_rng;
use macroquad_toolkit::ui::draw_ui_text;
use std::collections::HashSet;

impl Game {
    pub fn update(&mut self) {
        if is_key_pressed(KeyCode::F9) {
            self.show_ai_debug = !self.show_ai_debug;
        }

        self.draw_screen_background();
        if self.active_moment.is_some() {
            self.update_moment_timer();
            return;
        }
        self.tick += 1;

        let disciples_on_mission = self.collect_disciples_on_mission();

        self.scheduler.tick(
            &mut self.disciples,
            &self.data.buildings,
            &disciples_on_mission,
        );

        if self.tick % 60 == 0 {
            self.update_cultivation_tick(&disciples_on_mission);
            self.update_passive_income();
            self.process_herb_gardens();
            self.update_missions();
            self.update_world_sim();
        }

        self.update_season();

        if self.tick % 300 == 0 {
            self.update_world_evolution();
        }

        if self.tick % 600 == 0 {
            self.update_salary();
        }

        self.dispatch_state_update();
        self.update_moment_timer();
    }

    fn collect_disciples_on_mission(&self) -> HashSet<usize> {
        self.ongoing_missions
            .iter()
            .flat_map(|m| m.disciple_indices.iter().copied())
            .collect()
    }

    fn update_cultivation_tick(&mut self, disciples_on_mission: &HashSet<usize>) {
        // Update breakthrough readiness for disciples at threshold
        for (i, disciple) in self.disciples.iter_mut().enumerate() {
            if disciples_on_mission.contains(&i) || disciple.exp < disciple.exp_to_next_level {
                continue;
            }

            let old_readiness = disciple.breakthrough_readiness;
            disciple.update_readiness();

            if old_readiness == 0.0 && disciple.breakthrough_readiness > 0.0 {
                let realm_index = self
                    .data
                    .stages_order
                    .iter()
                    .position(|id| id == &disciple.realm)
                    .unwrap_or(0);
                let bottleneck = crate::engine::bottleneck::generate_bottleneck(
                    disciple,
                    &self.data,
                    realm_index,
                );
                if let Some(ref bn) = bottleneck {
                    let desc = bn.player_description(&self.data);
                    self.event_log.push(format!(
                        "{} feels the heavenly gate, but fate demands a trial first: {}",
                        disciple.name, desc
                    ));
                    disciple.breakthrough_bottleneck = Some(bn.clone());
                } else {
                    self.event_log.push(format!(
                        "{} stands before a heavenly gate. Open Disciple Scrolls to attempt breakthrough.",
                        disciple.name
                    ));
                }
            }
        }

        let (yard_multiplier, yard_score) = self
            .data
            .buildings
            .iter()
            .find(|b| b.building_type == BuildingType::TrainingYard)
            .map(|b| (b.get_cultivation_multiplier(), b.feng_shui_score))
            .unwrap_or((1.0, 0.0));

        let feng_shui_mod = yard_score / 100.0;
        let final_yard_multiplier = (yard_multiplier + feng_shui_mod).max(0.1);

        for (i, disciple) in self.disciples.iter_mut().enumerate() {
            if disciples_on_mission.contains(&i) {
                continue;
            }

            if disciple.is_injured() {
                let was_injured = disciple
                    .injury
                    .as_ref()
                    .map(|inj| inj.recovery_ticks_remaining)
                    .unwrap_or(0);
                disciple.heal_tick();

                if !disciple.is_injured() && was_injured > 0 {
                    self.event_log.push(format!(
                        "{} has recovered from their injuries.",
                        disciple.name
                    ));
                }
                continue;
            }

            let base_exp = Self::calculate_cultivation_exp(
                &self.data,
                &self.grid,
                disciple,
                final_yard_multiplier,
            );
            disciple.exp += base_exp;
        }
    }

    fn calculate_law_multiplier(
        data: &crate::data::loader::GameData,
        grid: &crate::data::grid::Grid,
        disciple: &crate::data::disciples::Disciple,
    ) -> (f32, u32) {
        let mut law_multiplier = 1.0;
        let mut extra_exp = 0u32;

        let Some(law_id) = &disciple.law_id else {
            return (law_multiplier, extra_exp);
        };
        let Some(law) = data.laws.get(law_id) else {
            return (law_multiplier, extra_exp);
        };

        let yard = data
            .buildings
            .iter()
            .find(|b| b.building_type == BuildingType::TrainingYard);
        if let Some(yard) = yard {
            if let Some(tile) = grid.get_tile(yard.x, yard.y) {
                let env_element = &tile.dominant_element;
                if *env_element == law.element {
                    law_multiplier += 0.5;
                } else if env_element.feeds() == law.element {
                    law_multiplier += 0.2;
                } else if env_element.suppresses() == law.element {
                    law_multiplier -= 0.5;
                }
            }
        }

        extra_exp += (disciple.attributes.body * law.stat_growth_modifiers.body) / 10;
        extra_exp += (disciple.attributes.mind * law.stat_growth_modifiers.mind) / 10;
        extra_exp += (disciple.attributes.spirit * law.stat_growth_modifiers.spirit) / 10;

        (law_multiplier, extra_exp)
    }

    fn calculate_cultivation_exp(
        data: &crate::data::loader::GameData,
        grid: &crate::data::grid::Grid,
        disciple: &crate::data::disciples::Disciple,
        final_yard_multiplier: f32,
    ) -> u32 {
        let mut base_exp = 1 + (disciple.attributes.spirit / 5);

        let (law_multiplier, law_extra_exp) = Self::calculate_law_multiplier(data, grid, disciple);
        base_exp += law_extra_exp;

        let trait_cultivation_mod: f32 = disciple
            .fate_traits
            .iter()
            .map(|t| t.cultivation_speed_modifier)
            .sum();

        let mut bloodline_mod = 0.0;
        if let Some(bloodline_id) = &disciple.bloodline.bloodline_id {
            if let Some(bloodline) = data.bloodlines.get(bloodline_id) {
                bloodline_mod = bloodline.passive_effects.cultivation_speed_modifier
                    * disciple.bloodline.effectiveness();
            }
        }

        let total_multiplier =
            (final_yard_multiplier * law_multiplier + trait_cultivation_mod + bloodline_mod)
                .max(0.1);

        (base_exp as f32 * total_multiplier) as u32
    }

    fn update_passive_income(&mut self) {
        if self.data.buildings.iter().any(|b| {
            b.building_type == BuildingType::SectHall && b.status == BuildingStatus::Active
        }) {
            self.spirit_stones += 1;
        }

        if let Some(garden) = self
            .data
            .buildings
            .iter()
            .find(|b| b.building_type == BuildingType::SpiritGarden)
        {
            let outer_count = self
                .disciples
                .iter()
                .filter(|d| d.rank == DiscipleRank::Outer)
                .count();
            if outer_count > 0 {
                let income = garden.get_passive_income();
                self.spirit_stones += income;

                if game_rng::chance(0.1) {
                    self.herbs += 1;
                }
            }
        }
    }

    fn update_missions(&mut self) {
        let mut completed_indices = Vec::new();
        for (i, mission) in self.ongoing_missions.iter_mut().enumerate() {
            mission.ticks_remaining = mission.ticks_remaining.saturating_sub(1);
            if mission.ticks_remaining == 0 {
                completed_indices.push(i);
            }
        }

        for index in completed_indices.into_iter().rev() {
            let mission = self.ongoing_missions.remove(index);
            let outcome = self.calculate_mission_outcome(mission);
            self.completed_missions.push(outcome);
            self.transition(StateTransition::ToMissionResolution);
        }
    }

    fn update_season(&mut self) {
        self.season_ticks = self.season_ticks.saturating_sub(1);
        if self.season_ticks == 0 {
            let old_season = self.current_season.clone();
            self.current_season = self.current_season.next();
            self.season_ticks = 3600;
            self.event_log.push(format!(
                "The season has changed from {} to {}.",
                old_season, self.current_season
            ));
            self.apply_herb_decay();
        }
    }

    fn update_world_evolution(&mut self) {
        for node in self.data.map_nodes.iter_mut() {
            if game_rng::chance(0.3) {
                node.corruption += 1;
                if node.corruption % 10 == 0 {
                    self.event_log.push(format!(
                        "Nodes are corrupting! {} danger increased.",
                        node.name
                    ));
                }
            }
        }
    }

    fn update_world_sim(&mut self) {
        let results = self.world_sim.tick(self.tick, &self.current_season);
        for result in results {
            self.handle_world_sim_result(result);
        }
    }

    fn update_salary(&mut self) {
        let inner_count = self
            .disciples
            .iter()
            .filter(|d| d.rank == DiscipleRank::Inner || d.rank == DiscipleRank::SectLeader)
            .count();
        if inner_count == 0 {
            return;
        }

        let salary_cost = inner_count as u32;
        if self.spirit_stones >= salary_cost {
            self.spirit_stones -= salary_cost;
        } else {
            self.spirit_stones = 0;
            self.event_log.push(
                "The treasury cannot pay inner-court stipends. Sect morale falls.".to_string(),
            );
        }
    }

    fn dispatch_state_update(&mut self) {
        let update_result = match &mut self.state {
            GameState::MainMenu(s) => s.update(),
            GameState::Intro(s) => s.update(&self.sect_name),
            GameState::SectBase(s) => s.update(
                &mut self.data,
                &mut self.grid,
                &self.textures,
                self.spirit_stones,
                self.herbs,
                self.influence,
                self.relics,
                &self.sect_name,
                &self.inventory,
                &self.unlocked_techs,
                &self.event_log,
                &self.ongoing_missions,
                &self.completed_missions,
                &self.completed_history,
                &self.disciples,
                &self.spirit_beasts,
                &self.current_season,
                self.season_ticks,
                &mut self.tutorial,
                &self.discovered_recipes,
            ),
            GameState::DiscipleRoster(s) => s.update(&self.data, &self.disciples, &self.inventory),
            GameState::WorldMap(s) => s.update(&self.data),
            GameState::MissionResolution(s) => s.update(&mut self.completed_missions),
            GameState::Library(s) => {
                s.update(&self.data, self.spirit_stones, &self.deceased_disciples)
            }
            GameState::MissionAssignment(s) => s.update(&self.data, &self.disciples),
            GameState::SectCreation(s) => s.update(),
            GameState::Tribulation(s) => s.update(&self.disciples),
            GameState::FactionScreen(s) => s.update(&self.world_sim),
            GameState::TradeScreen(s) => {
                s.update(&self.world_sim, self.spirit_stones, &self.inventory)
            }
        };

        if let Some(action) = update_result.action {
            self.execute_action(action);
        }
        if let Some(transition) = update_result.transition {
            self.transition(transition);
        }
    }

    /// Draw the background texture for the current screen state
    fn draw_screen_background(&self) {
        let bg_name = match &self.state {
            GameState::MainMenu(_) => "bg_main_menu",
            GameState::Intro(_) => "bg_main_menu",
            GameState::SectBase(_) => "bg_sect_base",
            GameState::DiscipleRoster(_) => "bg_roster",
            GameState::WorldMap(_) => "bg_world_map",
            GameState::MissionResolution(_) => "bg_mission_result",
            GameState::Library(_) => "bg_library",
            GameState::MissionAssignment(_) => "bg_mission_assign",
            GameState::SectCreation(_) => "bg_sect_creation",
            GameState::Tribulation(_) => "bg_tribulation",
            GameState::FactionScreen(_) => "bg_factions",
            GameState::TradeScreen(_) => "bg_trade",
        };

        self.textures.draw_background(bg_name);

        let overlay_alpha = match &self.state {
            GameState::MainMenu(_) => 0.1,
            GameState::Tribulation(_) => 0.05,
            GameState::SectBase(_) => 0.2,
            _ => 0.15,
        };
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, overlay_alpha),
        );
    }

    pub fn draw(&mut self) {
        if self.active_moment.is_some() {
            self.draw_active_moment();
            return;
        }
        match &mut self.state {
            GameState::MainMenu(s) => s.draw(&self.data, self.spirit_stones),
            GameState::Intro(s) => s.draw(),
            GameState::SectBase(s) => s.draw(&self.data, &self.grid, self.spirit_stones),
            GameState::DiscipleRoster(s) => s.draw(&self.data, &self.disciples, self.spirit_stones),
            GameState::WorldMap(s) => s.draw(&self.data, self.spirit_stones),
            GameState::MissionResolution(s) => s.draw(&self.data, self.spirit_stones),
            GameState::Library(s) => {
                s.draw(&self.data, self.spirit_stones, &self.deceased_disciples)
            }
            GameState::MissionAssignment(s) => {
                s.draw(&self.data, &self.disciples, self.spirit_stones)
            }
            GameState::SectCreation(s) => s.draw(&self.data),
            GameState::Tribulation(s) => s.draw(&self.data, &self.disciples),
            GameState::FactionScreen(s) => s.draw(&self.world_sim),
            GameState::TradeScreen(s) => s.draw(&self.world_sim),
        }

        if self.show_ai_debug {
            self.draw_ai_debug();
        }
    }

    fn draw_ai_debug(&self) {
        let panel_w = 420.0;
        let panel_h = screen_height() - 20.0;
        let panel_x = screen_width() - panel_w - 10.0;
        let panel_y = 10.0;

        draw_panel(
            Rect::new(panel_x, panel_y, panel_w, panel_h),
            Some("AI Debug (F9)"),
        );

        let mut y = panel_y + 55.0;
        draw_ui_text(
            &format!(
                "Assignments: {}  Reservations: {}",
                self.scheduler.assignment_count(),
                self.scheduler.reservation_count()
            ),
            panel_x + 15.0,
            y,
            FONT_SMALL_SIZE,
            TEXT_SECONDARY,
        );
        y += 25.0;

        for disciple in &self.disciples {
            if y > panel_y + panel_h - 120.0 {
                break;
            }

            let task_label = self
                .scheduler
                .get_assignment(disciple.id)
                .map(|a| format!("{} ({}t)", a.task.task_type, a.ticks_remaining))
                .unwrap_or_else(|| "Idle".to_string());

            draw_ui_text(
                &format!("{} [{}]", disciple.name, task_label),
                panel_x + 15.0,
                y,
                FONT_BODY_SIZE,
                TEXT_PRIMARY,
            );
            y += 22.0;

            let bar_w = panel_w - 30.0;
            let bar_h = 10.0;

            draw_ui_text(
                "Hunger",
                panel_x + 15.0,
                y + 10.0,
                FONT_SMALL_SIZE,
                TEXT_SECONDARY,
            );
            draw_progress_bar(
                Rect::new(panel_x + 85.0, y, bar_w - 70.0, bar_h),
                disciple.needs.hunger.current / disciple.needs.hunger.max,
                SECONDARY,
            );
            y += 16.0;

            draw_ui_text(
                "Rest",
                panel_x + 15.0,
                y + 10.0,
                FONT_SMALL_SIZE,
                TEXT_SECONDARY,
            );
            draw_progress_bar(
                Rect::new(panel_x + 85.0, y, bar_w - 70.0, bar_h),
                disciple.needs.rest.current / disciple.needs.rest.max,
                PRIMARY,
            );
            y += 16.0;

            draw_ui_text(
                "Qi",
                panel_x + 15.0,
                y + 10.0,
                FONT_SMALL_SIZE,
                TEXT_SECONDARY,
            );
            draw_progress_bar(
                Rect::new(panel_x + 85.0, y, bar_w - 70.0, bar_h),
                disciple.needs.qi.current / disciple.needs.qi.max,
                SUCCESS,
            );
            y += 16.0;

            draw_ui_text(
                "Morale",
                panel_x + 15.0,
                y + 10.0,
                FONT_SMALL_SIZE,
                TEXT_SECONDARY,
            );
            draw_progress_bar(
                Rect::new(panel_x + 85.0, y, bar_w - 70.0, bar_h),
                disciple.needs.morale.current / disciple.needs.morale.max,
                ACCENT,
            );
            y += 24.0;
        }
    }

    pub(super) fn transition(&mut self, transition: StateTransition) {
        self.state = match transition {
            StateTransition::ToMainMenu => GameState::MainMenu(MainMenuState::new()),
            StateTransition::ToIntro => GameState::Intro(IntroState::new()),
            StateTransition::ToSectBase => GameState::SectBase(SectBaseState::new()),
            StateTransition::ToDiscipleRoster => {
                GameState::DiscipleRoster(DiscipleRosterState::new())
            }
            StateTransition::ToWorldMap => GameState::WorldMap(WorldMapState::new()),
            StateTransition::ToMissionAssignment(desc) => {
                GameState::MissionAssignment(MissionAssignmentState::new(desc))
            }
            StateTransition::ToMissionResolution => {
                GameState::MissionResolution(MissionResolutionState::new())
            }
            StateTransition::ToLibrary => GameState::Library(LibraryState::new()),
            StateTransition::ToSectCreation => GameState::SectCreation(SectCreationState::new()),
            StateTransition::ToTribulation(trib_state, disciple_idx) => {
                GameState::Tribulation(TribulationEncounterState::new(trib_state, disciple_idx))
            }
            StateTransition::ToFactionScreen => GameState::FactionScreen(FactionScreenState::new()),
            StateTransition::ToTradeScreen => GameState::TradeScreen(TradeScreenState::new()),
        };
    }
}
