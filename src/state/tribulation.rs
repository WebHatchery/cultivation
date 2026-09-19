//! A trial keeps health and the next strike visible while the chronicle is paged.
use crate::data::{disciples::Disciple, loader::GameData};
use crate::engine::{actions::Action, tribulation::TribulationState};
use crate::state::{StateTransition, UpdateResult};
use crate::ui::{components::*, theme::*, workspace as ux};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub struct TribulationEncounterState {
    pub tribulation: TribulationState,
    pub disciple_index: usize,
    page: usize,
}
impl TribulationEncounterState {
    pub fn new(tribulation: TribulationState, disciple_index: usize) -> Self {
        Self {
            tribulation,
            disciple_index,
            page: 0,
        }
    }
    pub fn update(&mut self, disciples: &[Disciple]) -> UpdateResult {
        let finished = self.tribulation.is_finished;
        let leave = ux::header_to("Heavenly Tribulation", "Each strike advances the trial. Watch the disciple's life flame. Chronicle: latest first.", if finished { "Record outcome" } else { "" });
        let Some(disciple) = disciples.get(self.disciple_index) else {
            if leave {
                return UpdateResult::new().with_transition(StateTransition::ToSectBase);
            }
            draw_ui_text(
                "The disciple is no longer present. Return to the sect.",
                24.0,
                150.0,
                FONT_BODY_SIZE,
                WARNING,
            );
            return UpdateResult::new();
        };
        let mut area = ux::content();
        area.h -= 64.0;
        let (vitals, record) = ux::split(area);
        draw_panel(vitals, Some(&disciple.name));
        let hp = (self.tribulation.disciple_hp.max(0) as f32
            / self.tribulation.disciple_max_hp.max(1) as f32)
            .clamp(0.0, 1.0);
        draw_ui_text(
            &format!(
                "Life: {} / {}",
                self.tribulation.disciple_hp.max(0),
                self.tribulation.disciple_max_hp
            ),
            vitals.x + 16.0,
            vitals.y + 70.0,
            FONT_BODY_SIZE,
            TEXT_PRIMARY,
        );
        draw_progress_bar(
            Rect::new(vitals.x + 16.0, vitals.y + 88.0, vitals.w - 32.0, 18.0),
            hp,
            if hp < 0.3 { FAILURE } else { SUCCESS },
        );
        draw_ui_text(
            &format!(
                "Strikes endured: {} / {}",
                self.tribulation.current_wave, self.tribulation.config.total_waves
            ),
            vitals.x + 16.0,
            vitals.y + 138.0,
            FONT_SMALL_SIZE,
            TEXT_SECONDARY,
        );
        let stage = Rect::new(vitals.x, vitals.y + 154.0, vitals.w, vitals.h - 154.0);
        ux::clipped(stage, || {
            let center = vec2(stage.x + stage.w / 2.0, stage.y + stage.h / 2.0);
            let pulse = (get_time() as f32 * 2.4).sin() * 0.5 + 0.5;
            draw_celestial_rings(center, pulse, finished);
            draw_lightning_walls(center, self.tribulation.current_wave, pulse);
            draw_disciple_silhouette(center, disciple, self.tribulation.survived, finished);
        });
        draw_panel(
            record,
            Some(if finished {
                if self.tribulation.survived {
                    "Trial survived"
                } else {
                    "Disciple fallen"
                }
            } else {
                "Storm chronicle"
            }),
        );
        let mut rows = vec![
            self.tribulation.config.name.clone(),
            self.tribulation.config.description.clone(),
        ];
        rows.extend(self.tribulation.log.iter().rev().cloned());
        if self.tribulation.log.is_empty() {
            rows.push("Tap Endure next strike when ready to begin.".to_owned());
        }
        ux::text_pages(
            Rect::new(
                record.x + 16.0,
                record.y + 56.0,
                record.w - 32.0,
                record.h - 72.0,
            ),
            &rows,
            &mut self.page,
        );
        let clicked = ux::button(
            Rect::new(area.x, screen_height() - 72.0, area.w, 48.0),
            if finished {
                "Record outcome and return"
            } else {
                "Endure next strike"
            },
            true,
            true,
        );
        if finished && (clicked || leave) {
            return UpdateResult::new()
                .with_action(Action::ResolveTribulation {
                    disciple_idx: self.disciple_index,
                    survived: self.tribulation.survived,
                })
                .with_transition(StateTransition::ToSectBase);
        }
        if !finished && clicked {
            self.tribulation.process_wave(disciple);
            self.page = 0;
        }
        UpdateResult::new()
    }
    pub fn draw(&self, _data: &GameData, _disciples: &[Disciple]) {}
}

fn draw_celestial_rings(center: Vec2, pulse: f32, finished: bool) {
    let color = if finished { PRIMARY } else { WARNING };
    for i in 0..5 {
        let r = 62.0 + i as f32 * 42.0 + pulse * 10.0;
        draw_circle_lines(
            center.x,
            center.y,
            r,
            1.5,
            Color::new(color.r, color.g, color.b, 0.28 - i as f32 * 0.035),
        );
    }
    draw_circle(
        center.x,
        center.y,
        84.0 + pulse * 8.0,
        Color::new(color.r, color.g, color.b, 0.08),
    );
}

fn draw_lightning_walls(center: Vec2, wave: u32, pulse: f32) {
    let count = wave.max(1).min(9);
    for i in 0..count {
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let x = center.x + side * (92.0 + i as f32 * 19.0);
        let top = center.y - 210.0;
        let mid = center.y - 84.0 + pulse * 18.0;
        let bottom = center.y + 118.0;
        let color = Color::new(0.64, 0.80, 1.0, 0.20 + i as f32 * 0.025);
        draw_line(x, top, x + side * 22.0, mid, 2.0, color);
        draw_line(x + side * 22.0, mid, x - side * 12.0, bottom, 2.0, color);
    }
}

fn draw_disciple_silhouette(center: Vec2, disciple: &Disciple, survived: bool, finished: bool) {
    let aura = if finished && !survived {
        FAILURE
    } else if finished {
        SUCCESS
    } else {
        SECONDARY
    };

    draw_circle(
        center.x,
        center.y + 10.0,
        58.0,
        Color::new(aura.r, aura.g, aura.b, 0.18),
    );
    draw_circle_lines(
        center.x,
        center.y + 10.0,
        66.0,
        2.0,
        Color::new(aura.r, aura.g, aura.b, 0.52),
    );
    draw_circle(
        center.x,
        center.y - 22.0,
        18.0,
        Color::new(0.92, 0.82, 0.62, 0.94),
    );
    draw_rectangle(
        center.x - 18.0,
        center.y - 5.0,
        36.0,
        62.0,
        Color::new(0.08, 0.05, 0.04, 0.92),
    );
    draw_line(
        center.x - 30.0,
        center.y + 12.0,
        center.x + 30.0,
        center.y + 12.0,
        4.0,
        aura,
    );

    let dims = measure_ui_text(&disciple.name, None, FONT_BODY_SIZE as u16, 1.0);
    draw_ui_text(
        &disciple.name,
        center.x - dims.width / 2.0,
        center.y + 100.0,
        FONT_BODY_SIZE,
        TEXT_PRIMARY,
    );
}
