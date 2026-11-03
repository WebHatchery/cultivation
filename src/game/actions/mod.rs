mod buildings;
mod crafting;
mod diplomacy;
mod disciples;
mod items;
mod missions;

use super::Game;
use crate::engine::actions::Action;
use crate::engine::random;
use crate::engine::world_sim::WorldSim;
use crate::game::moments::MomentKind;
use crate::state::StateTransition;

impl Game {
    pub(super) fn execute_action(&mut self, action: Action) {
        match action {
            Action::UpgradeBuilding(building_type) => self.handle_upgrade_building(building_type),
            Action::RecruitDisciple => self.handle_recruit_disciple(),
            Action::PromoteDisciple(idx) => self.handle_promote_disciple(idx),
            Action::AttemptBreakthrough(idx) => self.handle_attempt_breakthrough_action(idx),
            Action::ResolveTribulation {
                disciple_idx,
                survived,
            } => self.handle_resolve_tribulation(disciple_idx, survived),
            Action::StartMission(mission_desc, disciple_indices) => {
                self.handle_start_mission(mission_desc, disciple_indices)
            }
            Action::ClaimRewards(outcome) => self.handle_claim_rewards(outcome),
            Action::SaveGame => self.handle_save_game(),
            Action::LoadGame => self.handle_load_game(),
            Action::ToggleFullscreen => self.display_settings.begin_preview(),
            Action::AssignLaw(disciple_idx, law_id) => self.handle_assign_law(disciple_idx, law_id),
            Action::CraftItem(recipe_id) => self.handle_craft_item(recipe_id),
            Action::UseItem(item_id, disciple_idx) => self.handle_use_item(item_id, disciple_idx),
            Action::EquipItem(item_id, disciple_idx) => {
                self.handle_equip_item(item_id, disciple_idx)
            }
            Action::UnequipItem(slot, disciple_idx) => self.handle_unequip_item(slot, disciple_idx),
            Action::RepairItem(item_id, disciple_idx) => {
                self.handle_repair_item(item_id, disciple_idx)
            }
            Action::ResearchTech(tech_id) => self.handle_research_tech(tech_id),
            Action::RepairBuilding(id) => self.handle_repair_building(id),
            Action::ConstructBuilding(b_type, x, y) => self.handle_construct_building(b_type, x, y),
            Action::StartNewGame(name) => self.handle_start_new_game(name),
            Action::PlantHerb(building_id, plot_index, herb_id) => {
                self.plant_herb(building_id, plot_index, &herb_id);
            }
            Action::AssignDiscipleToBuilding(building_id, disciple_id) => {
                self.assign_disciple_to_building(building_id, disciple_id);
            }
            Action::ProcessDryingPavilion(building_id, herb_id) => {
                self.process_drying(building_id, &herb_id);
            }
            Action::SetGreenhouseInfusion(building_id, element) => {
                self.set_greenhouse_infusion(building_id, element);
            }
            Action::SendDiplomat { faction_id, action } => {
                self.handle_send_diplomat(faction_id, action)
            }
            Action::RespondToEvent {
                event_id,
                choice_idx,
            } => self.handle_respond_to_event(event_id, choice_idx),
            Action::RecruitSpiritBeast(def_id) => self.handle_recruit_spirit_beast(def_id),
            Action::EquipBeastItem(beast_id, item_id) => {
                self.handle_equip_beast_item(beast_id, item_id)
            }
        }
    }

    fn handle_save_game(&mut self) {
        self.save();
        self.event_log.push("Game saved successfully.".to_string());
    }

    fn handle_load_game(&mut self) {
        let loaded_game = self.load();
        if let Some(mut game) = loaded_game {
            let loaded_buildings = game.data.buildings.clone();
            game.data = self.data.clone();
            game.data.buildings = loaded_buildings;
            *self = game;
            self.event_log.push("Game loaded successfully.".to_string());
            self.active_moment = None;
        }
    }

    fn handle_start_new_game(&mut self, name: String) {
        use crate::data::buildings::BuildingType;
        use crate::data::disciples::DiscipleRank;
        use crate::engine::proc_gen::generate_disciple;

        let data = self.data.clone();

        let mut leader = generate_disciple(&data);
        leader.name = "Patriarch".to_string();
        leader.rank = DiscipleRank::SectLeader;

        if let Some(survivor_trait) = data.fate_traits.iter().find(|t| t.name == "Survivor") {
            leader.fate_traits.retain(|t| t.name != "Survivor");
            leader.fate_traits.push(survivor_trait.clone());
        }

        self.disciples = vec![leader];
        self.sect_name = name;
        self.spirit_stones = 50;
        self.herbs = 10;
        self.influence = 0;
        self.relics = 0;
        self.inventory.clear();
        self.unlocked_techs.clear();
        self.discovered_recipes = vec![
            "recipe_healing_pill".to_string(),
            "recipe_iron_sword".to_string(),
        ];
        self.ongoing_missions.clear();
        self.completed_missions.clear();
        self.completed_history.clear();
        self.deceased_disciples.clear();
        self.event_log = vec!["The sect has fallen... We must rebuild.".to_string()];
        self.tutorial = crate::state::TutorialState::new();
        self.tick = 0;
        self.current_season = crate::data::herbs::Season::Spring;
        self.season_ticks = 3600;
        self.grid = crate::data::grid::Grid::new(20, 20);

        self.world_sim = WorldSim::new(
            self.data.factions.clone(),
            self.data.economy_nodes.clone(),
            self.data.trade_routes.clone(),
            self.data.world_events.clone(),
            self.data.balance.clone(),
        );

        self.data.buildings.clear();

        let mut sect_hall = crate::data::buildings::Building::new(BuildingType::SectHall);
        sect_hall.id = random::next_u64();
        sect_hall.x = 2048;
        sect_hall.y = 2048;
        sect_hall.element = self
            .data
            .building_definitions
            .get(&BuildingType::SectHall)
            .map(|d| d.element.clone())
            .unwrap_or_default();
        sect_hall.status = crate::data::buildings::BuildingStatus::Ruined;
        if let Some(def) = self.data.building_definitions.get(&BuildingType::SectHall) {
            sect_hall.repair_cost = def.repair_cost;
        }
        self.data.buildings.push(sect_hall);

        self.transition(StateTransition::ToIntro);
    }
}
