//! Seed real management states for repeatable, non-persistent UI captures.
use super::*;
use crate::engine::actions::Action;
use crate::state::{
    mission_assignment::MissionAssignmentState, mission_resolution::MissionResolutionState,
    roster::DiscipleRosterState, tribulation::TribulationEncounterState,
};
impl Game {
    pub(super) fn prepare_management_capture(&mut self, scene: &str) -> bool {
        if !matches!(
            scene,
            "roster"
                | "roster_orders"
                | "dispatch"
                | "dispatch_team"
                | "dispatch_review"
                | "report"
                | "tribulation"
        ) {
            return false;
        }
        self.execute_action(Action::StartNewGame("Fallen Peak".to_owned()));
        self.active_moment = None;
        match scene {
            "roster" | "roster_orders" => {
                let mut state = DiscipleRosterState::new();
                state.prepare_capture(if scene == "roster_orders" { 2 } else { 0 });
                self.state = GameState::DiscipleRoster(state);
            }
            "dispatch" | "dispatch_team" | "dispatch_review" => {
                if let Some(mission) = self.data.missions.first() {
                    let mut state = MissionAssignmentState::new(mission.description.clone());
                    state.prepare_capture(if scene == "dispatch_team" {
                        1
                    } else if scene == "dispatch_review" {
                        2
                    } else {
                        0
                    });
                    self.state = GameState::MissionAssignment(state);
                }
            }
            "report" => {
                self.completed_missions.push(MissionOutcome {
                    success: true,
                    mission_name: "A return from Serene Forest".to_owned(),
                    description:
                        "The disciples returned with herbs and a safer route through the foothills."
                            .to_owned(),
                    logs: vec![
                        "The expedition found a sheltered path.".to_owned(),
                        "A disciple gathered medicinal herbs near the river.".to_owned(),
                        "The team returned through the mountain gate.".to_owned(),
                    ],
                    rewards: crate::data::missions::MissionRewards {
                        spirit_stones: 25,
                        herbs: 5,
                        disciple_exp: 10,
                        ..Default::default()
                    },
                    disciple_indices: vec![0],
                });
                self.state = GameState::MissionResolution(MissionResolutionState::new());
            }
            _ => {
                if let Some(disciple) = self.disciples.first() {
                    let trial = crate::engine::tribulation::TribulationState::new(
                        crate::engine::tribulation::TribulationType::GoldenCore,
                        disciple,
                    );
                    self.state = GameState::Tribulation(TribulationEncounterState::new(trial, 0));
                }
            }
        }
        true
    }
}
