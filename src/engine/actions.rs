use crate::data::buildings::BuildingType;
use crate::data::relations::DiplomaticAction;

#[derive(Debug)]
pub enum Action {
    UpgradeBuilding(BuildingType),
    RecruitDisciple,
    PromoteDisciple(usize),           // Disciple Index
    StartMission(String, Vec<usize>), // Mission Description, Disciple Indices
    ClaimRewards(crate::data::missions::MissionOutcome),
    AssignLaw(usize, String), // Disciple Index, Law ID
    CraftItem(String),        // Recipe ID
    UseItem(String, usize),   // Item ID, Disciple Index
    EquipItem(String, usize), // Item ID, Disciple Index
    UnequipItem(crate::data::items::EquipmentSlot, usize), // Slot, Disciple Index
    RepairItem(String, usize), // Item ID, Disciple Index — repairs equipped item durability
    ResearchTech(String),     // Tech ID
    RepairBuilding(u64),      // Building ID
    ConstructBuilding(BuildingType, i32, i32), // Type, X, Y
    SaveGame,
    LoadGame,
    ToggleFullscreen,
    StartNewGame(String), // Sect Name
    // Herb system actions
    PlantHerb(u64, usize, String), // Building ID, Plot Index, Herb ID
    AssignDiscipleToBuilding(u64, Option<u64>), // Building ID, Disciple ID (None to unassign)
    ProcessDryingPavilion(u64, String), // Building ID, Herb ID to dry
    SetGreenhouseInfusion(u64, Option<crate::data::elements::Element>), // Building ID, Element
    // Diplomacy and world simulation actions
    SendDiplomat {
        faction_id: String,
        action: DiplomaticAction,
    },
    RespondToEvent {
        event_id: String,
        choice_idx: usize,
    },
    // Breakthrough action (player-controlled)
    AttemptBreakthrough(usize), // Disciple Index
    ResolveTribulation {
        disciple_idx: usize,
        survived: bool,
    },
    // Spirit beast actions
    RecruitSpiritBeast(String),  // Definition ID
    EquipBeastItem(u64, String), // Beast ID, Item ID
}
