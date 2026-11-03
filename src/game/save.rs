use super::Game;
use crate::data::grid::Grid;
use crate::engine::scheduler::Scheduler;
use crate::engine::world_sim::WorldSim;
use crate::save::{storage, SaveData, SavedTutorialState, SAVE_VERSION};

impl Game {
    pub(super) fn save(&self) {
        use crate::engine::world_sim::SavedWorldSim;

        let save_data = SaveData {
            version: SAVE_VERSION,
            grid: Some(self.grid.clone()),
            sect_name: self.sect_name.clone(),
            spirit_stones: self.spirit_stones,
            herbs: self.herbs,
            influence: self.influence,
            relics: self.relics,
            inventory: self.inventory.clone(),
            unlocked_techs: self.unlocked_techs.clone(),
            disciples: self.disciples.clone(),
            deceased_disciples: self.deceased_disciples.clone(),
            spirit_beasts: self.spirit_beasts.clone(),
            buildings: self.data.buildings.clone(),
            ongoing_missions: self.ongoing_missions.clone(),
            completed_missions: self.completed_missions.clone(),
            completed_history: self.completed_history.clone(),
            tick: self.tick,
            current_season: self.current_season.clone(),
            season_ticks: self.season_ticks,
            tutorial: SavedTutorialState::from_tutorial(&self.tutorial),
            world_sim: Some(SavedWorldSim::from(&self.world_sim)),
            scheduler: Some(self.scheduler.to_saved()),
            discovered_recipes: self.discovered_recipes.clone(),
        };

        if let Err(e) = storage::save(&save_data) {
            eprintln!("Failed to save: {}", e);
        }
    }

    pub(super) fn load(&mut self) -> Option<Self> {
        match storage::load() {
            Ok(save_data) => {
                let world_sim = if let Some(saved_world_sim) = save_data.world_sim {
                    WorldSim::from_saved(saved_world_sim, self.data.world_events.clone())
                } else {
                    WorldSim::new(
                        self.data.factions.clone(),
                        self.data.economy_nodes.clone(),
                        self.data.trade_routes.clone(),
                        self.data.world_events.clone(),
                        self.data.balance.clone(),
                    )
                };

                let scheduler = if let Some(saved_scheduler) = save_data.scheduler {
                    Scheduler::from_saved(saved_scheduler, self.data.ai_scheduler.clone())
                } else {
                    Scheduler::new(self.data.ai_scheduler.clone())
                };

                let textures = std::mem::take(&mut self.textures);
                let fonts = std::mem::take(&mut self.fonts);
                let display_settings = std::mem::take(&mut self.display_settings);

                let mut new_game = Self {
                    state: crate::state::GameState::SectBase(
                        crate::state::sect_base::SectBaseState::new(),
                    ),
                    data: self.data.clone(),
                    grid: save_data.grid.unwrap_or_else(|| Grid::new(20, 20)),
                    sect_name: save_data.sect_name,
                    spirit_stones: save_data.spirit_stones,
                    herbs: save_data.herbs,
                    influence: save_data.influence,
                    relics: save_data.relics,
                    inventory: save_data.inventory,
                    unlocked_techs: save_data.unlocked_techs,
                    disciples: save_data.disciples,
                    deceased_disciples: save_data.deceased_disciples,
                    spirit_beasts: save_data.spirit_beasts,
                    ongoing_missions: save_data.ongoing_missions,
                    completed_missions: save_data.completed_missions,
                    completed_history: save_data.completed_history,
                    event_log: vec!["Game loaded successfully.".to_string()],
                    discovered_recipes: save_data.discovered_recipes,
                    tutorial: save_data.tutorial.to_tutorial(),
                    tick: save_data.tick,
                    current_season: save_data.current_season,
                    season_ticks: save_data.season_ticks,
                    world_sim,
                    scheduler,
                    show_ai_debug: false,
                    textures,
                    fonts,
                    active_moment: None,
                    display_settings,
                    display_notice: None,
                };

                // Restore building states
                new_game.data.buildings = save_data.buildings;

                Some(new_game)
            }
            Err(e) => {
                eprintln!("Failed to load: {}", e);
                None
            }
        }
    }
}
