use crate::data::buildings::{BuildingStatus, BuildingType};
use crate::data::disciples::{Disciple, DiscipleRank};
use crate::data::grid::Grid;
use crate::data::herbs::Season;
use crate::data::history::DeceasedDisciple;
use crate::data::loader::GameData;
use crate::data::missions::{MissionOutcome, OngoingMission};
use crate::data::spirit_beasts::SpiritBeast;
use crate::engine::proc_gen::generate_disciple;
use crate::engine::scheduler::Scheduler;
use crate::engine::world_sim::WorldSim;
use crate::state::{
    main_menu::MainMenuState, sect_base::SectBaseState, world_map::WorldMapState, GameState,
};
use crate::ui::{FontManager, TextureManager};

mod actions;
mod breakthrough;
mod herbs;
mod items;
mod missions;
mod moments;
mod save;
mod update;
mod world;

use self::moments::MomentOverlay;

const FOUNDATION_TRIAL_MISSION: &str = "Foundation Trial (Solo)";

/// Result of a breakthrough attempt
pub enum BreakthroughResult {
    Success,
    Failure,                                                  // Died
    Injured,                                                  // Survived but didn't advance
    Tribulation(crate::engine::tribulation::TribulationType), // Needs tribulation
    Blocked,                                                  // Blocked by requirements
}

pub struct Game {
    pub state: GameState,
    pub data: GameData,
    pub grid: Grid,
    pub sect_name: String,
    pub spirit_stones: u32,
    pub herbs: u32,
    pub influence: u32,
    pub relics: u32,
    pub inventory: std::collections::HashMap<String, u32>, // ItemID -> Count
    pub unlocked_techs: Vec<String>,
    pub disciples: Vec<Disciple>,
    pub deceased_disciples: Vec<DeceasedDisciple>,
    pub spirit_beasts: Vec<SpiritBeast>,
    pub ongoing_missions: Vec<OngoingMission>,
    pub completed_missions: Vec<MissionOutcome>,
    pub completed_history: Vec<String>, // List of descriptions of successfully completed missions
    pub event_log: Vec<String>,
    pub discovered_recipes: Vec<String>,
    pub tutorial: crate::state::TutorialState,
    tick: u64,
    /// Current season for herb growth
    pub current_season: Season,
    /// Ticks until next season change (3600 ticks = 1 minute at 60 fps)
    pub season_ticks: u32,
    /// World simulation for factions, diplomacy, economy, and events
    pub world_sim: WorldSim,
    /// AI scheduler for disciple tasks
    pub scheduler: Scheduler,
    /// Toggle for AI debug overlay
    show_ai_debug: bool,
    /// Texture manager for all game graphics
    pub textures: TextureManager,
    /// Font manager for custom fonts
    pub fonts: FontManager,
    /// Short-lived dramatic presentation for major sect moments
    pub(in crate::game) active_moment: Option<MomentOverlay>,
}

impl Game {
    pub async fn new() -> Self {
        let mut data = GameData::load_async()
            .await
            .expect("Failed to load game data");
        data.buildings.clear(); // User requested blank map

        // Scenario: Survivors of the Fallen Sect
        // 1. The Patriarch (Sect Leader)
        let mut leader = generate_disciple(&data);
        leader.name = "Patriarch".to_string();
        leader.rank = DiscipleRank::SectLeader;

        // Give Patriarch the Survivor trait - they cannot die, only suffer setbacks
        if let Some(survivor_trait) = data.fate_traits.iter().find(|t| t.name == "Survivor") {
            // Remove any conflicting traits and add Survivor
            leader.fate_traits.retain(|t| t.name != "Survivor");
            leader.fate_traits.push(survivor_trait.clone());
        }

        // 2. No starting workers (User requested "do not start with disciples")
        let disciples = vec![leader];

        let grid = Grid::new(20, 20);

        // Initialize world simulation
        let world_sim = WorldSim::new(
            data.factions.clone(),
            data.economy_nodes.clone(),
            data.trade_routes.clone(),
            data.world_events.clone(),
            data.balance.clone(),
        );

        // Clone ai_scheduler config before moving data
        let ai_scheduler_config = data.ai_scheduler.clone();

        // Load all textures
        let mut textures = TextureManager::new();
        textures.load_all().await;
        println!(
            "Game: Loaded {} textures ({} errors)",
            textures.texture_count(),
            textures.error_count()
        );

        // Load fonts
        let mut fonts = FontManager::new();
        fonts.load_all().await;

        Self {
            state: GameState::MainMenu(MainMenuState::new()),
            data,
            grid,
            sect_name: "Unnamed Sect".to_string(), // Initial placeholder
            spirit_stones: 50,                     // Reduced to 50 as Sect Hall is free
            herbs: 10,                             // Some supplies
            influence: 0,
            relics: 0,
            inventory: std::collections::HashMap::new(),
            unlocked_techs: Vec::new(),
            disciples,
            deceased_disciples: Vec::new(),
            spirit_beasts: Vec::new(),
            ongoing_missions: Vec::new(),
            completed_missions: Vec::new(),
            completed_history: Vec::new(),
            event_log: vec!["The sect has fallen... We must rebuild.".to_string()],
            discovered_recipes: vec![
                "recipe_healing_pill".to_string(),
                "recipe_iron_sword".to_string(),
            ],
            tutorial: crate::state::TutorialState::new(),
            tick: 0,
            current_season: Season::Spring,
            season_ticks: 3600, // 1 minute per season at 60 fps
            world_sim,
            scheduler: Scheduler::new(ai_scheduler_config),
            show_ai_debug: false,
            textures,
            fonts,
            active_moment: None,
        }
    }

    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "intro" => {
                self.execute_action(crate::engine::actions::Action::StartNewGame(
                    "Fallen Peak Sect".to_string(),
                ));
            }
            "sectbase" | "hall" | "doctrine" | "annals" | "construction" => {
                // Seed a real fresh campaign so the scene matches actual play
                // (ruined Sect Hall, tutorial step 0, starting resources).
                self.execute_action(crate::engine::actions::Action::StartNewGame(
                    "Fallen Peak Sect".to_string(),
                ));
                self.transition(crate::state::StateTransition::ToSectBase);
                self.active_moment = None;
                if scene == "construction" {
                    self.unlocked_techs.push("sect_administration".to_owned());
                }
                if scene != "sectbase" {
                    if let Some(hall) = self
                        .data
                        .buildings
                        .iter_mut()
                        .find(|b| b.building_type == BuildingType::SectHall)
                    {
                        hall.status = BuildingStatus::Active;
                        if let GameState::SectBase(state) = &mut self.state {
                            state.prepare_capture(scene, hall.id);
                        }
                    }
                }
            }
            "worldmap" => self.state = GameState::WorldMap(WorldMapState::new()),
            "trade" => {
                self.state =
                    GameState::TradeScreen(crate::state::trade_screen::TradeScreenState::new())
            }
            "factions" => {
                self.state = GameState::FactionScreen(
                    crate::state::faction_screen::FactionScreenState::new(),
                )
            }
            "library" => {
                self.state = GameState::Library(crate::state::library::LibraryState::new())
            }
            _ => {
                // Default: the boot state, the main menu.
                self.state = GameState::MainMenu(MainMenuState::new());
            }
        }
    }

    fn get_population_capacity(&self) -> u32 {
        self.data
            .buildings
            .iter()
            .filter(|b| b.status == BuildingStatus::Active)
            .map(|b| match b.building_type {
                BuildingType::SectHall => b.get_max_disciples(),
                BuildingType::Dormitory => b.get_dorm_capacity(),
                _ => 0,
            })
            .sum()
    }
}
