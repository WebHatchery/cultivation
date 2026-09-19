//! A return report prioritizes the outcome and rewards, with the full record one tab away.
use crate::data::{
    loader::GameData,
    missions::{MissionOutcome, MissionRewards},
};
use crate::engine::actions::Action;
use crate::state::{StateTransition, UpdateResult};
use crate::ui::{components::*, theme::*, workspace as ux};
use macroquad::prelude::*;

pub struct MissionResolutionState {
    current_outcome: Option<MissionOutcome>,
    tab: usize,
    page: usize,
}
impl MissionResolutionState {
    pub fn new() -> Self {
        Self {
            current_outcome: None,
            tab: 0,
            page: 0,
        }
    }
    pub fn update(&mut self, completed: &mut Vec<MissionOutcome>) -> UpdateResult {
        if self.current_outcome.is_none() {
            self.current_outcome = completed.pop();
            self.tab = 0;
            self.page = 0;
        }
        let Some(outcome) = &self.current_outcome else {
            return UpdateResult::new().with_transition(StateTransition::ToSectBase);
        };
        // The report must be recorded before leaving; the header action records it too.
        let record = ux::header_to(
            "Dispatch Return",
            if outcome.success {
                "The expedition returned successfully. Review the rewards and record the outcome."
            } else {
                "The expedition suffered a setback. Review the record before continuing."
            },
            "Record outcome",
        );
        let mut area = ux::content();
        if ux::tabs(
            area,
            &["Outcome & rewards", "Journey record"],
            &mut self.tab,
        ) {
            self.page = 0;
        }
        area.y += 60.0;
        area.h -= 124.0;
        let rows = if self.tab == 0 {
            let mut rows = vec![outcome.mission_name.clone(), outcome.description.clone()];
            rows.extend(
                reward_entries(&outcome.rewards)
                    .into_iter()
                    .map(|(label, value, _)| format!("{}: {}", label, value)),
            );
            rows.extend(
                outcome
                    .rewards
                    .items
                    .iter()
                    .map(|(id, count)| format!("Item: {} x{}", id.replace('_', " "), count)),
            );
            rows.extend(
                outcome
                    .rewards
                    .recipe_discoveries
                    .iter()
                    .map(|id| format!("Recipe discovered: {}", id.replace('_', " "))),
            );
            rows
        } else {
            outcome.logs.clone()
        };
        ux::text_pages(area, &rows, &mut self.page);
        if ux::button(
            Rect::new(area.x, screen_height() - 72.0, area.w, 48.0),
            "Record outcome and continue",
            true,
            true,
        ) || record
        {
            let action = Action::ClaimRewards(outcome.clone());
            self.current_outcome = None;
            return UpdateResult::new().with_action(action);
        }
        UpdateResult::new()
    }
    pub fn draw(&self, _data: &GameData, _spirit_stones: u32) {}
}

fn reward_entries(rewards: &MissionRewards) -> Vec<(&'static str, String, Color)> {
    let mut entries = Vec::new();
    if rewards.spirit_stones > 0 {
        entries.push(("Spirit Stones", rewards.spirit_stones.to_string(), PRIMARY));
    }
    if rewards.disciple_exp > 0 {
        entries.push(("Insight", format!("{} qi", rewards.disciple_exp), SECONDARY));
    }
    if rewards.herbs > 0 {
        entries.push(("Herbs", rewards.herbs.to_string(), SUCCESS));
    }
    if rewards.influence > 0 {
        entries.push(("Prestige", rewards.influence.to_string(), WARNING));
    }
    if rewards.relics > 0 {
        entries.push(("Relics", rewards.relics.to_string(), ACCENT));
    }
    if !rewards.items.is_empty() {
        let total: u32 = rewards.items.iter().map(|(_, count)| *count).sum();
        entries.push(("Materials", total.to_string(), TEXT_HIGHLIGHT));
    }
    entries
}
