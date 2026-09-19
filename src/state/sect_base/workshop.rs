//! Workshop recipes reserve the right pane for requirements and an explicit craft action.
use super::*;
use crate::engine::crafting::{calculate_success_chance, CraftingContext};
use crate::ui::workspace as ux;
use macroquad_toolkit::ui::draw_ui_text;
impl SectBaseState {
    pub(super) fn draw_crafting_modal(
        &mut self,
        data: &GameData,
        building_type: &BuildingType,
        stones: u32,
        herbs: u32,
        inventory: &HashMap<String, u32>,
        disciples: &[Disciple],
        discovered: &[String],
    ) -> Option<UpdateResult> {
        if ux::header_to(
            "Sect Workshop",
            "Review materials and success chance before attempting a formula.",
            "Back to hall",
        ) {
            self.crafting_modal_open = false;
            return None;
        }
        let building = data
            .buildings
            .iter()
            .find(|b| b.building_type == *building_type && b.status == BuildingStatus::Active)?;
        let mut area = ux::content();
        if ux::tabs(area, &["Recipes", "Attendant"], &mut self.facility_tab) {
            self.facility_page = 0;
            self.facility_detail_page = 0;
        }
        area.y += 60.0;
        area.h -= 60.0;
        if self.facility_tab == 1 {
            return self.draw_attendants(area, building, data, disciples);
        }
        let mut recipes: Vec<_> = data
            .recipes
            .iter()
            .filter(|r| r.required_building == *building_type && discovered.contains(&r.id))
            .collect();
        recipes.sort_by(|a, b| a.name.cmp(&b.name));
        if recipes.is_empty() {
            draw_ui_text(
                "No formulas have been recovered for this workshop.",
                area.x,
                area.y + 30.0,
                FONT_BODY_SIZE,
                TEXT_SECONDARY,
            );
            return None;
        }
        let (list, detail) = ux::split(area);
        self.facility_selected = self.facility_selected.min(recipes.len() - 1);
        let range = ux::page(list, recipes.len(), 60.0, &mut self.facility_page);
        for (row, index) in range.enumerate() {
            if ux::button(
                Rect::new(list.x, list.y + row as f32 * 60.0, list.w, 48.0),
                &recipes[index].name,
                self.facility_selected == index,
                true,
            ) {
                self.facility_selected = index;
                self.facility_detail_page = 0;
            }
        }
        let recipe = recipes[self.facility_selected];
        let context = CraftingContext {
            building_level: building.level,
            building_element: building.element.clone(),
            disciple_mind: building
                .assigned_disciple
                .and_then(|id| disciples.iter().find(|d| d.id == id))
                .map(|d| d.attributes.mind),
        };
        let chance = calculate_success_chance(recipe, &context);
        let mut rows = vec![
            recipe.name.clone(),
            format!(
                "Success chance: {}% / Output: {}",
                chance, recipe.output_amount
            ),
            "An unsuccessful attempt may consume materials.".to_owned(),
        ];
        let mut affordable = true;
        for (id, needed) in &recipe.ingredients {
            let owned = match id.as_str() {
                "spirit_stones" => stones,
                "herbs" => herbs,
                _ => inventory.get(id).copied().unwrap_or(0),
            };
            affordable &= owned >= *needed;
            let name = data
                .items
                .get(id)
                .map(|item| item.name.clone())
                .unwrap_or_else(|| id.replace('_', " "));
            rows.push(format!("{}: need {} / have {}", name, needed, owned));
        }
        if let Some(item) = data.items.get(&recipe.output_item_id) {
            rows.push(item.description.clone());
        }
        draw_panel(detail, Some("Formula review"));
        ux::text_pages(
            Rect::new(
                detail.x + 16.0,
                detail.y + 56.0,
                detail.w - 32.0,
                detail.h - 128.0,
            ),
            &rows,
            &mut self.facility_detail_page,
        );
        if ux::button(
            Rect::new(
                detail.x + 16.0,
                detail.y + detail.h - 60.0,
                detail.w - 32.0,
                44.0,
            ),
            if affordable {
                "Attempt formula"
            } else {
                "Materials missing"
            },
            true,
            affordable,
        ) {
            return Some(UpdateResult::new().with_action(Action::CraftItem(recipe.id.clone())));
        }
        None
    }
}
