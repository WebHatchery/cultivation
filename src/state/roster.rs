//! Disciples are selected by their roster index, independent of filters and pages.
use crate::data::disciples::{Disciple, DiscipleRank};
use crate::data::loader::GameData;
use crate::state::{StateTransition, UpdateResult};
use crate::ui::{components::*, theme::*, workspace as ux};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;
mod details;
mod modals;

pub struct DiscipleRosterState {
    selected: Option<usize>,
    filter: usize,
    page: usize,
    tab: usize,
    detail_page: usize,
    law_modal_open: bool,
    item_modal_open: bool,
    equip_modal_open: bool,
    modal_page: usize,
    modal_selected: usize,
    equipment_tab: usize,
}
impl DiscipleRosterState {
    pub fn new() -> Self {
        Self {
            selected: None,
            filter: 0,
            page: 0,
            tab: 0,
            detail_page: 0,
            law_modal_open: false,
            item_modal_open: false,
            equip_modal_open: false,
            modal_page: 0,
            modal_selected: 0,
            equipment_tab: 0,
        }
    }
    pub fn prepare_capture(&mut self, tab: usize) {
        self.tab = tab;
    }
    pub fn update(
        &mut self,
        data: &GameData,
        disciples: &[Disciple],
        inventory: &std::collections::HashMap<String, u32>,
    ) -> UpdateResult {
        if let Some(index) = self.selected.filter(|index| *index < disciples.len()) {
            if self.law_modal_open {
                return self
                    .draw_law_modal(data, index)
                    .unwrap_or_else(UpdateResult::new);
            }
            if self.item_modal_open {
                return self
                    .draw_item_modal(data, index, inventory)
                    .unwrap_or_else(UpdateResult::new);
            }
            if self.equip_modal_open {
                return self
                    .draw_equip_modal(data, index, inventory, disciples)
                    .unwrap_or_else(UpdateResult::new);
            }
        }
        if ux::header(
            "Disciple Scrolls",
            "Select a disciple, review readiness, then choose an order.",
        ) {
            return UpdateResult::new().with_transition(StateTransition::ToSectBase);
        }
        let mut area = ux::content();
        if ux::tabs(
            area,
            &["All", "Outer", "Inner+", "Injured", "Ready"],
            &mut self.filter,
        ) {
            self.page = 0;
            self.selected = None;
        }
        area.y += 60.0;
        area.h -= 60.0;
        let filtered: Vec<_> = disciples
            .iter()
            .enumerate()
            .filter(|(_, d)| match self.filter {
                1 => d.rank == DiscipleRank::Outer,
                2 => d.rank != DiscipleRank::Outer,
                3 => d.is_injured(),
                4 => d.can_attempt_breakthrough(),
                _ => true,
            })
            .collect();
        if !filtered
            .iter()
            .any(|(index, _)| Some(*index) == self.selected)
        {
            self.selected = filtered.first().map(|(index, _)| *index);
        }
        let (list, detail) = ux::split(area);
        let range = ux::page(list, filtered.len(), 74.0, &mut self.page);
        for (row, position) in range.enumerate() {
            let (index, disciple) = filtered[position];
            let y = list.y + row as f32 * 74.0;
            if ux::button(
                Rect::new(list.x, y, list.w, 46.0),
                &disciple.name,
                self.selected == Some(index),
                true,
            ) {
                self.selected = Some(index);
                self.detail_page = 0;
            }
            let status = if disciple.is_injured() {
                "Recovering"
            } else if disciple.can_attempt_breakthrough() {
                "Ready for breakthrough"
            } else {
                "Cultivating"
            };
            draw_ui_text(
                status,
                list.x + 10.0,
                y + 64.0,
                FONT_SMALL_SIZE,
                if disciple.is_injured() {
                    WARNING
                } else {
                    TEXT_SECONDARY
                },
            );
        }
        if filtered.is_empty() {
            draw_wrapped_text(
                "No disciples match this filter.",
                list.x,
                list.y + 30.0,
                list.w,
                FONT_BODY_SIZE,
                TEXT_SECONDARY,
            );
        }
        if let Some(index) = self.selected {
            if let Some(disciple) = disciples.get(index) {
                if let Some(result) = self.draw_disciple_record(detail, data, disciple, index) {
                    return result;
                }
            }
        }
        UpdateResult::new()
    }
    pub fn draw(&self, _data: &GameData, _disciples: &[Disciple], _spirit_stones: u32) {}
}
