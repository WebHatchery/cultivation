//! The dispatch board separates open expeditions, ongoing teams and returned reports.
use super::*;
use crate::ui::workspace as ux;
use macroquad_toolkit::ui::draw_ui_text;
impl SectBaseState {
    pub(super) fn draw_mission_board(
        &mut self,
        data: &GameData,
        ongoing: &[OngoingMission],
        completed: &[MissionOutcome],
        history: &[String],
    ) -> Option<UpdateResult> {
        if ux::header(
            "Mission Board",
            "Choose an expedition, check teams away, or record returned reports.",
        ) {
            self.view = SectView::Map;
            return None;
        }
        let mut area = ux::content();
        if ux::tabs(
            area,
            &["Available", "Away", "Returned"],
            &mut self.mission_tab,
        ) {
            self.mission_page = 0;
        }
        area.y += 60.0;
        area.h -= 60.0;
        if self.mission_tab == 0 {
            let missions: Vec<_> = data
                .missions
                .iter()
                .filter(|mission| {
                    !ongoing
                        .iter()
                        .any(|entry| entry.mission.description == mission.description)
                        && !completed
                            .iter()
                            .any(|entry| entry.description == mission.description)
                        && (mission.repeatable || !history.contains(&mission.description))
                })
                .collect();
            let range = ux::page(area, missions.len(), 88.0, &mut self.mission_page);
            for (row, index) in range.enumerate() {
                let mission = missions[index];
                let y = area.y + row as f32 * 88.0;
                if ux::button(
                    Rect::new(area.x, y, area.w, 48.0),
                    &mission.description,
                    false,
                    true,
                ) {
                    return Some(UpdateResult::new().with_transition(
                        StateTransition::ToMissionAssignment(mission.description.clone()),
                    ));
                }
                draw_ui_text(
                    &format!(
                        "{} / Danger {} / {}",
                        mission.mission_type,
                        mission.danger_level,
                        crate::data::time::days_label(mission.duration)
                    ),
                    area.x + 12.0,
                    y + 72.0,
                    FONT_SMALL_SIZE,
                    TEXT_SECONDARY,
                );
            }
            if missions.is_empty() {
                draw_ui_text(
                    "No expeditions are available. Check Away or Returned.",
                    area.x,
                    area.y + 30.0,
                    FONT_BODY_SIZE,
                    TEXT_SECONDARY,
                );
            }
        } else if self.mission_tab == 1 {
            let mut rows: Vec<_> = ongoing
                .iter()
                .map(|mission| {
                    format!(
                        "{} / {} remaining / {} disciples",
                        mission.mission.description,
                        crate::data::time::days_label(mission.ticks_remaining),
                        mission.disciple_indices.len()
                    )
                })
                .collect();
            if rows.is_empty() {
                rows.push("No teams are away.".to_owned());
            }
            ux::text_pages(area, &rows, &mut self.mission_page);
        } else {
            if completed.is_empty() {
                draw_ui_text(
                    "No reports await your seal.",
                    area.x,
                    area.y + 30.0,
                    FONT_BODY_SIZE,
                    TEXT_SECONDARY,
                );
            } else {
                draw_ui_text(
                    &format!("{} reports await review.", completed.len()),
                    area.x,
                    area.y + 30.0,
                    FONT_BODY_SIZE,
                    PRIMARY,
                );
                if ux::button(
                    Rect::new(area.x, area.y + 64.0, area.w, 48.0),
                    "Review returned expeditions",
                    true,
                    true,
                ) {
                    return Some(
                        UpdateResult::new().with_transition(StateTransition::ToMissionResolution),
                    );
                }
            }
        }
        None
    }
}
