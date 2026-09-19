//! Guardian records prioritize bond health, then statistics and equipment.
use super::*;
use crate::ui::workspace as ux;
use macroquad_toolkit::ui::draw_ui_text;
impl SectBaseState {
    pub(super) fn draw_guardians(
        &mut self,
        data: &GameData,
        beasts: &[SpiritBeast],
        inventory: &HashMap<String, u32>,
    ) -> Option<UpdateResult> {
        if self.selected_beast_index.is_none() && !beasts.is_empty() {
            self.selected_beast_index = Some(0);
        }
        if self.beast_equip_modal_open {
            if let Some(beast) = self.selected_beast_index.and_then(|i| beasts.get(i)) {
                return self.draw_beast_gear(data, beast, inventory);
            }
            self.beast_equip_modal_open = false;
        }
        if ux::header(
            "Mountain Guardians",
            "Inspect each guardian's bond, condition, and equipment.",
        ) {
            self.view = SectView::Map;
            return None;
        }
        let area = ux::content();
        if beasts.is_empty() {
            draw_wrapped_text("No spirit beasts are bonded to the sect yet. Their records will appear here when they join you.", area.x, area.y + 30.0, area.w, FONT_BODY_SIZE, TEXT_SECONDARY);
            return None;
        }
        let (list, detail) = ux::split(area);
        let range = ux::page(list, beasts.len(), 64.0, &mut self.beast_page);
        for (row, index) in range.enumerate() {
            if ux::button(
                Rect::new(list.x, list.y + row as f32 * 64.0, list.w, 48.0),
                &beasts[index].name,
                self.selected_beast_index == Some(index),
                true,
            ) {
                self.selected_beast_index = Some(index);
                self.beast_detail_page = 0;
            }
        }
        if let Some(beast) = self.selected_beast_index.and_then(|i| beasts.get(i)) {
            draw_panel(detail, Some(&beast.name));
            let mut rows = vec![
                format!("Loyalty {} / Hunger {}", beast.loyalty, beast.hunger),
                format!(
                    "{} / {} / {} aspect",
                    beast.species, beast.tier, beast.element
                ),
                format!(
                    "Health {} / Attack {} / Defense {} / Speed {} / Qi {}",
                    beast.stats.health,
                    beast.stats.attack,
                    beast.stats.defense,
                    beast.stats.speed,
                    beast.stats.qi
                ),
            ];
            rows.extend(
                beast
                    .traits
                    .iter()
                    .map(|name| format!("Trait: {}", name.replace('_', " "))),
            );
            rows.extend(
                beast
                    .abilities
                    .iter()
                    .map(|name| format!("Ability: {}", name.replace('_', " "))),
            );
            for slot in [
                crate::data::spirit_beasts::BeastEquipmentSlot::Collar,
                crate::data::spirit_beasts::BeastEquipmentSlot::Harness,
                crate::data::spirit_beasts::BeastEquipmentSlot::Talisman,
                crate::data::spirit_beasts::BeastEquipmentSlot::Relic,
            ] {
                let name = beast
                    .equipment
                    .iter()
                    .find(|item| item.slot == slot)
                    .and_then(|item| data.beast_equipment_definitions.get(&item.item_id))
                    .map(|item| item.name.as_str())
                    .unwrap_or("Empty");
                rows.push(format!("{}: {}", slot, name));
            }
            ux::text_pages(
                Rect::new(
                    detail.x + 16.0,
                    detail.y + 56.0,
                    detail.w - 32.0,
                    detail.h - 128.0,
                ),
                &rows,
                &mut self.beast_detail_page,
            );
            if ux::button(
                Rect::new(
                    detail.x + 16.0,
                    detail.y + detail.h - 60.0,
                    detail.w - 32.0,
                    44.0,
                ),
                "Equip guardian gear",
                true,
                true,
            ) {
                self.beast_equip_modal_open = true;
                self.beast_item_page = 0;
            }
        }
        None
    }
    fn draw_beast_gear(
        &mut self,
        data: &GameData,
        beast: &SpiritBeast,
        inventory: &HashMap<String, u32>,
    ) -> Option<UpdateResult> {
        if ux::header_to(
            "Guardian Gear",
            "Review slot and bonuses. Items above the guardian's tier cannot be equipped.",
            "Back to bond",
        ) {
            self.beast_equip_modal_open = false;
            return None;
        }
        let area = ux::content();
        let mut items: Vec<_> = data
            .beast_equipment_definitions
            .values()
            .filter(|item| inventory.get(&item.id).copied().unwrap_or(0) > 0)
            .collect();
        items.sort_by(|a, b| a.name.cmp(&b.name));
        let range = ux::page(area, items.len(), 100.0, &mut self.beast_item_page);
        for (row, index) in range.enumerate() {
            let item = items[index];
            let y = area.y + row as f32 * 100.0;
            if ux::button(
                Rect::new(area.x, y, area.w, 48.0),
                &format!(
                    "Equip {} / {} slot / Requires {}",
                    item.name, item.slot, item.tier_required
                ),
                true,
                beast.can_equip(item),
            ) {
                self.beast_equip_modal_open = false;
                return Some(
                    UpdateResult::new()
                        .with_action(Action::EquipBeastItem(beast.id, item.id.clone())),
                );
            }
            draw_ui_text(
                &format!(
                    "Health {} / Attack {} / Defense {} / Speed {} / Qi {}",
                    item.stats.health,
                    item.stats.attack,
                    item.stats.defense,
                    item.stats.speed,
                    item.stats.qi
                ),
                area.x + 12.0,
                y + 76.0,
                FONT_BODY_SIZE,
                TEXT_SECONDARY,
            );
        }
        if items.is_empty() {
            draw_ui_text(
                "No guardian equipment is stored in the sect.",
                area.x,
                area.y + 30.0,
                FONT_BODY_SIZE,
                TEXT_SECONDARY,
            );
        }
        None
    }
}
