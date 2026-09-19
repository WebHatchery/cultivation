//! Item and law choices show their effect before committing an order.
use super::*;
use crate::engine::actions::Action;
type Choice = (String, String, Action);
impl DiscipleRosterState {
    pub(super) fn draw_law_modal(&mut self, data: &GameData, index: usize) -> Option<UpdateResult> {
        if ux::header_to(
            "Assign Cultivation Law",
            "Review a teaching before assigning it to this disciple.",
            "Back to record",
        ) {
            self.law_modal_open = false;
            return None;
        }
        let mut laws: Vec<_> = data.laws.values().collect();
        laws.sort_by(|a, b| a.name.cmp(&b.name));
        let choices = laws
            .iter()
            .map(|law| {
                (
                    law.name.clone(),
                    format!("Element: {}. {}", law.element, law.description),
                    Action::AssignLaw(index, law.id.clone()),
                )
            })
            .collect();
        self.draw_choices(ux::content(), choices, "Assign law")
    }
    pub(super) fn draw_item_modal(
        &mut self,
        data: &GameData,
        index: usize,
        inventory: &std::collections::HashMap<String, u32>,
    ) -> Option<UpdateResult> {
        if ux::header_to(
            "Use Supplies",
            "Review the effect. Using a supply consumes one item.",
            "Back to record",
        ) {
            self.item_modal_open = false;
            return None;
        }
        let mut choices: Vec<Choice> = inventory
            .iter()
            .filter(|(_, count)| **count > 0)
            .filter_map(|(id, count)| {
                let item = data.items.get(id)?;
                if item.equipment.is_some() {
                    return None;
                }
                Some((
                    format!("{} x{}", item.name, count),
                    item.description.clone(),
                    Action::UseItem(id.clone(), index),
                ))
            })
            .collect();
        choices.sort_by(|a, b| a.0.cmp(&b.0));
        self.draw_choices(ux::content(), choices, "Use one item")
    }
    pub(super) fn draw_equip_modal(
        &mut self,
        data: &GameData,
        index: usize,
        inventory: &std::collections::HashMap<String, u32>,
        disciples: &[Disciple],
    ) -> Option<UpdateResult> {
        if ux::header_to(
            "Artifact Binding",
            "Inspect available artifacts or unbind an equipped item.",
            "Back to record",
        ) {
            self.equip_modal_open = false;
            return None;
        }
        let mut area = ux::content();
        if ux::tabs(area, &["Available", "Equipped"], &mut self.equipment_tab) {
            self.modal_page = 0;
            self.modal_selected = 0;
            self.detail_page = 0;
        }
        area.y += 60.0;
        area.h -= 60.0;
        let mut choices: Vec<Choice> = if self.equipment_tab == 0 {
            inventory
                .iter()
                .filter(|(_, count)| **count > 0)
                .filter_map(|(id, count)| {
                    let item = data.items.get(id)?;
                    let equipment = item.equipment.as_ref()?;
                    Some((
                        format!("{} x{}", item.name, count),
                        format!("Slot: {:?}. {}", equipment.slot, item.description),
                        Action::EquipItem(id.clone(), index),
                    ))
                })
                .collect()
        } else {
            disciples
                .get(index)?
                .equipment
                .iter()
                .map(|(slot, id)| {
                    let item = data.items.get(id);
                    (
                        format!(
                            "{:?}: {}",
                            slot,
                            item.map(|item| item.name.as_str()).unwrap_or(id)
                        ),
                        item.map(|item| item.description.clone())
                            .unwrap_or_default(),
                        Action::UnequipItem(slot.clone(), index),
                    )
                })
                .collect()
        };
        choices.sort_by(|a, b| a.0.cmp(&b.0));
        self.draw_choices(
            area,
            choices,
            if self.equipment_tab == 0 {
                "Bind artifact"
            } else {
                "Unbind artifact"
            },
        )
    }
    fn draw_choices(
        &mut self,
        area: Rect,
        choices: Vec<Choice>,
        action_label: &str,
    ) -> Option<UpdateResult> {
        if choices.is_empty() {
            draw_wrapped_text("Nothing is available in this category. Return to the record or choose another category.", area.x, area.y + 32.0, area.w, FONT_BODY_SIZE, TEXT_SECONDARY);
            return None;
        }
        self.modal_selected = self.modal_selected.min(choices.len() - 1);
        let (list, detail) = ux::split(area);
        let range = ux::page(list, choices.len(), 58.0, &mut self.modal_page);
        for (row, index) in range.enumerate() {
            if ux::button(
                Rect::new(list.x, list.y + row as f32 * 58.0, list.w, 48.0),
                &choices[index].0,
                self.modal_selected == index,
                true,
            ) {
                self.modal_selected = index;
                self.detail_page = 0;
            }
        }
        draw_panel(detail, Some("Review order"));
        ux::text_pages(
            Rect::new(
                detail.x + 16.0,
                detail.y + 56.0,
                detail.w - 32.0,
                detail.h - 132.0,
            ),
            &[
                choices[self.modal_selected].0.clone(),
                choices[self.modal_selected].1.clone(),
            ],
            &mut self.detail_page,
        );
        if ux::button(
            Rect::new(
                detail.x + 16.0,
                detail.y + detail.h - 62.0,
                detail.w - 32.0,
                46.0,
            ),
            action_label,
            true,
            true,
        ) {
            self.law_modal_open = false;
            self.item_modal_open = false;
            self.equip_modal_open = false;
            return choices
                .into_iter()
                .nth(self.modal_selected)
                .map(|choice| UpdateResult::new().with_action(choice.2));
        }
        None
    }
}
