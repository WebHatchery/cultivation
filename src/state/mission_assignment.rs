//! Dispatch follows briefing, team selection and review without competing columns.
use crate::data::{
    disciples::Disciple,
    loader::GameData,
    missions::{Mission, RelevantStat},
};
use crate::engine::actions::Action;
use crate::state::{StateTransition, UpdateResult};
use crate::ui::{components::*, theme::*, workspace as ux};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub struct MissionAssignmentState {
    pub mission_description: String,
    selected_disciples: Vec<usize>,
    step: usize,
    page: usize,
}
impl MissionAssignmentState {
    pub fn new(mission_description: String) -> Self {
        Self {
            mission_description,
            selected_disciples: Vec::new(),
            step: 0,
            page: 0,
        }
    }
    pub fn prepare_capture(&mut self, step: usize) {
        self.step = step;
        if step == 2 {
            self.selected_disciples = vec![0];
        }
    }
    pub fn update(&mut self, data: &GameData, disciples: &[Disciple]) -> UpdateResult {
        if ux::header(
            "Dispatch Beyond the Gate",
            "Read the brief, choose up to three disciples, then review the order.",
        ) {
            return UpdateResult::new().with_transition(StateTransition::ToSectBase);
        }
        let mut area = ux::content();
        if ux::tabs(
            area,
            &["1. Briefing", "2. Team", "3. Review"],
            &mut self.step,
        ) {
            self.page = 0;
        }
        area.y += 60.0;
        area.h -= 124.0;
        let mission = data
            .missions
            .iter()
            .find(|mission| mission.description == self.mission_description);
        let Some(mission) = mission else {
            draw_wrapped_text(
                "This dispatch is no longer available. Return to the Mission Board.",
                area.x,
                area.y + 30.0,
                area.w,
                FONT_BODY_SIZE,
                WARNING,
            );
            return UpdateResult::new();
        };
        self.selected_disciples
            .retain(|index| disciples.get(*index).is_some_and(|d| !d.is_injured()));
        match self.step {
            0 => ux::text_pages(area, &[
                mission.description.clone(),
                format!("Type: {} / Primary attribute: {}", mission.mission_type, mission.mission_type.get_relevant_stat()),
                format!("Danger: {} / Duration: {}", mission.danger_level, crate::data::time::days_label(mission.duration)),
                "Choose disciples with a strong primary attribute. Injured disciples cannot depart. A strong team improves the prospect, but does not guarantee a safe return.".to_owned(),
            ], &mut self.page),
            1 => self.draw_team(area, mission, disciples),
            _ => {
                let stat = team_mission_stat(&self.selected_disciples, Some(mission), disciples);
                let required = mission_required_stat(Some(mission));
                let names = self.selected_disciples.iter().filter_map(|index| disciples.get(*index)).map(|d| d.name.as_str()).collect::<Vec<_>>().join(", ");
                ux::text_pages(area, &[
                    mission.description.clone(), format!("Team: {}", if names.is_empty() { "Choose at least one disciple" } else { &names }),
                    format!("Team merit: {} / Threshold: {}", stat, required),
                    mission_prospect(stat, required).to_owned(),
                    format!("Danger {} / Away for {}", mission.danger_level, crate::data::time::days_label(mission.duration)),
                ], &mut self.page);
            }
        }
        let footer = Rect::new(area.x, screen_height() - 72.0, area.w, 48.0);
        if self.step > 0
            && ux::button(
                Rect::new(footer.x, footer.y, 150.0, 48.0),
                "Previous step",
                false,
                true,
            )
        {
            self.step -= 1;
            self.page = 0;
            return UpdateResult::new();
        }
        let ready = !self.selected_disciples.is_empty();
        let label = match self.step {
            0 => "Choose team",
            1 => "Review order",
            _ => "Send disciples",
        };
        if ux::button(
            Rect::new(footer.x + footer.w - 220.0, footer.y, 220.0, 48.0),
            label,
            true,
            self.step == 0 || ready,
        ) {
            if self.step < 2 {
                self.step += 1;
                self.page = 0;
            } else {
                return UpdateResult::new()
                    .with_action(Action::StartMission(
                        self.mission_description.clone(),
                        self.selected_disciples.clone(),
                    ))
                    .with_transition(StateTransition::ToSectBase);
            }
        }
        UpdateResult::new()
    }
    fn draw_team(&mut self, rect: Rect, mission: &Mission, disciples: &[Disciple]) {
        draw_ui_text(
            &format!(
                "{} / 3 chosen / Tap a selected disciple to remove them",
                self.selected_disciples.len()
            ),
            rect.x,
            rect.y + 24.0,
            FONT_BODY_SIZE,
            PRIMARY,
        );
        let list = Rect::new(rect.x, rect.y + 44.0, rect.w, rect.h - 44.0);
        let range = ux::page(list, disciples.len(), 66.0, &mut self.page);
        for (row, index) in range.enumerate() {
            let disciple = &disciples[index];
            let selected = self.selected_disciples.contains(&index);
            let label = format!(
                "{} / {} {} / {}",
                disciple.name,
                mission.mission_type.get_relevant_stat(),
                disciple_mission_stat(Some(mission), disciple),
                if disciple.is_injured() {
                    "Injured"
                } else if selected {
                    "Selected"
                } else {
                    "Available"
                }
            );
            if ux::button(
                Rect::new(list.x, list.y + row as f32 * 66.0, list.w, 52.0),
                &label,
                selected,
                !disciple.is_injured() && (selected || self.selected_disciples.len() < 3),
            ) {
                if selected {
                    self.selected_disciples.retain(|entry| *entry != index);
                } else {
                    self.selected_disciples.push(index);
                }
            }
        }
    }
    pub fn draw(&self, _data: &GameData, _disciples: &[Disciple], _spirit_stones: u32) {}
}

fn disciple_mission_stat(mission: Option<&Mission>, disciple: &Disciple) -> u32 {
    match mission.map(|m| m.mission_type.get_relevant_stat()) {
        Some(RelevantStat::Body) => disciple.attributes.body,
        Some(RelevantStat::Mind) => disciple.attributes.mind,
        Some(RelevantStat::Spirit) => disciple.attributes.spirit,
        None => {
            (disciple.attributes.body + disciple.attributes.mind + disciple.attributes.spirit) / 3
        }
    }
}

fn team_mission_stat(selected: &[usize], mission: Option<&Mission>, disciples: &[Disciple]) -> u32 {
    selected
        .iter()
        .filter_map(|idx| disciples.get(*idx))
        .map(|disciple| disciple_mission_stat(mission, disciple))
        .sum()
}

fn mission_required_stat(mission: Option<&Mission>) -> u32 {
    mission
        .map(|m| 18 + m.danger_level.saturating_mul(14))
        .unwrap_or(36)
}

fn mission_prospect(team_stat: u32, required: u32) -> &'static str {
    if team_stat == 0 {
        return "No names have been written on the dispatch order.";
    }
    if team_stat >= required + 28 {
        "The omens favor this expedition. The sect may gain more than it risks."
    } else if team_stat >= required {
        "The order is sound, but danger still waits beyond the gate."
    } else {
        "This order asks too much of the chosen disciples. Injury or failure is likely."
    }
}
