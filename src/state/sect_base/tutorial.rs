use super::*;
use macroquad_toolkit::ui::draw_ui_text;

impl SectBaseState {
    /// Register a rect to be highlighted while the given tutorial step is active.
    pub(super) fn register_tutorial_target(&mut self, step: usize, rect: Rect) {
        if self.tutorial_step_active == Some(step) {
            self.tutorial_targets.push(rect);
        }
    }

    /// Pulsing gold outlines around this frame's registered tutorial targets.
    pub(super) fn draw_tutorial_highlights(&self) {
        if self.tutorial_targets.is_empty() {
            return;
        }
        let pulse = 0.5 + 0.5 * ((get_time() as f32) * 3.2).sin();
        let alpha = 0.45 + 0.45 * pulse;
        let expand = 3.0 + 3.0 * pulse;
        for r in &self.tutorial_targets {
            draw_rectangle_lines(
                r.x - expand,
                r.y - expand,
                r.w + expand * 2.0,
                r.h + expand * 2.0,
                3.0,
                Color::new(TEXT_HIGHLIGHT.r, TEXT_HIGHLIGHT.g, TEXT_HIGHLIGHT.b, alpha),
            );
        }
    }

    pub(super) fn draw_tutorial_overlay(
        &self,
        screen_w: f32,
        header_h: f32,
        tutorial: &mut TutorialState,
    ) {
        let total_steps = 5;
        let (title, body) = self.get_tutorial_step_text(tutorial.step);
        let header = format!(
            "First Decrees {}/{}",
            (tutorial.step + 1).min(total_steps),
            total_steps
        );

        let rect = Rect::new(16.0, header_h + 4.0, screen_w - 32.0, 104.0);

        draw_panel(rect, Some(&header));

        if draw_button(
            Rect::new(rect.x + rect.w - 148.0, rect.y + 8.0, 64.0, 44.0),
            "Hide",
            false,
        ) {
            tutorial.hidden = true;
        }
        if draw_button(
            Rect::new(rect.x + rect.w - 76.0, rect.y + 8.0, 64.0, 44.0),
            "Skip",
            false,
        ) {
            tutorial.active = false;
        }

        draw_ui_text(
            title,
            rect.x + 16.0,
            rect.y + 64.0,
            FONT_BODY_SIZE,
            TEXT_HIGHLIGHT,
        );
        draw_wrapped_text(
            body,
            rect.x + 16.0,
            rect.y + 88.0,
            rect.w - 32.0,
            FONT_SMALL_SIZE,
            TEXT_SECONDARY,
        );
    }

    pub(super) fn update_tutorial_progress(
        &self,
        tutorial: &mut TutorialState,
        data: &GameData,
        unlocked_techs: &[String],
        disciples: &[Disciple],
        ongoing_missions: &[OngoingMission],
        completed_history: &[String],
    ) {
        if !tutorial.active {
            return;
        }

        let total_steps = 5;
        while tutorial.step < total_steps
            && self.is_tutorial_step_complete(
                tutorial.step,
                data,
                unlocked_techs,
                disciples,
                ongoing_missions,
                completed_history,
            )
        {
            tutorial.step += 1;
        }

        if tutorial.step >= total_steps {
            tutorial.active = false;
        }
    }

    fn is_tutorial_step_complete(
        &self,
        step: usize,
        data: &GameData,
        unlocked_techs: &[String],
        disciples: &[Disciple],
        ongoing_missions: &[OngoingMission],
        completed_history: &[String],
    ) -> bool {
        match step {
            0 => data.buildings.iter().any(|b| {
                b.building_type == BuildingType::SectHall && b.status == BuildingStatus::Active
            }),
            1 => unlocked_techs.iter().any(|t| t == "sect_administration"),
            2 => data
                .buildings
                .iter()
                .any(|b| b.building_type == BuildingType::MissionBoard),
            3 => disciples.len() > 1,
            4 => !ongoing_missions.is_empty() || !completed_history.is_empty(),
            _ => true,
        }
    }

    fn get_tutorial_step_text(&self, step: usize) -> (&'static str, &'static str) {
        match step {
            0 => (
                "Restore the Sect Hall",
                "Select the ruined Sect Hall and choose Restore Hall (50 SS).",
            ),
            1 => (
                "Recover the Dispatch Doctrine",
                "Open the Sect Hall, press Recover Doctrine, and learn Sect Administration (free).",
            ),
            2 => (
                "Raise the Mission Board",
                "Press Raise Halls and place the Mission Board on the mountain map.",
            ),
            3 => (
                "Accept a Disciple",
                "Select the Sect Hall and press Accept Disciple to bring in help.",
            ),
            4 => (
                "Issue a Dispatch",
                "Open the Mission Board, assign a team, and send them beyond the gate.",
            ),
            _ => (
                "Mandate Established",
                "The first patriarch decrees are complete.",
            ),
        }
    }
}
