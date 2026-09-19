//! Facility workspaces keep production, staffing and upgrades separate.
use super::*;
use crate::data::buildings::Building;
use crate::data::disciples::DiscipleRank;
use crate::ui::workspace as ux;
use macroquad_toolkit::ui::draw_ui_text;
impl SectBaseState {
    pub(super) fn draw_facility(
        &mut self,
        building: &Building,
        data: &GameData,
        inventory: &HashMap<String, u32>,
        disciples: &[Disciple],
        season: &Season,
        stones: u32,
    ) -> Option<UpdateResult> {
        if let Some(plot) = self.herb_planting_modal {
            return self.draw_seed_selection(building, data, season, plot);
        }
        if ux::header(
            &building.building_type.to_string(),
            "Review production, appoint an attendant, or improve this hall.",
        ) {
            self.view = SectView::Map;
            return None;
        }
        let mut area = ux::content();
        let greenhouse = building.building_type == BuildingType::Greenhouse;
        let labels: &[&str] = if greenhouse {
            &["Production", "Attendant", "Array", "Details"]
        } else {
            &["Production", "Attendant", "Details"]
        };
        if ux::tabs(area, labels, &mut self.facility_tab) {
            self.facility_page = 0;
        }
        area.y += 60.0;
        area.h -= 60.0;
        match self.facility_tab {
            0 => {
                if matches!(
                    building.building_type,
                    BuildingType::HerbGarden | BuildingType::Greenhouse
                ) {
                    return self.draw_plots(area, building, data);
                }
                return self.draw_stores(area, building, data, inventory);
            }
            1 => return self.draw_attendants(area, building, data, disciples),
            2 if greenhouse => {
                let elements = [
                    None,
                    Some(Element::Wood),
                    Some(Element::Fire),
                    Some(Element::Water),
                    Some(Element::Metal),
                    Some(Element::Earth),
                ];
                let range = ux::page(area, elements.len(), 54.0, &mut self.facility_page);
                for (row, index) in range.enumerate() {
                    let element = &elements[index];
                    let label = element
                        .as_ref()
                        .map(|e| format!("{} / Matching herbs ignore the season", e))
                        .unwrap_or_else(|| "Extinguish array / Follow natural seasons".to_owned());
                    if ux::button(
                        Rect::new(area.x, area.y + row as f32 * 54.0, area.w, 46.0),
                        &label,
                        building.infused_element == *element,
                        true,
                    ) {
                        return Some(UpdateResult::new().with_action(
                            Action::SetGreenhouseInfusion(building.id, element.clone()),
                        ));
                    }
                }
            }
            _ => {
                let description = data
                    .building_definitions
                    .get(&building.building_type)
                    .map(|d| d.description.clone())
                    .unwrap_or_default();
                let rows = vec![
                    description,
                    format!(
                        "Grade {} / Aspect {} / Feng Shui {:.1}",
                        building.level, building.element, building.feng_shui_score
                    ),
                    format!("Season: {} / Treasury: {} stones", season, stones),
                ];
                ux::text_pages(
                    Rect::new(area.x, area.y, area.w, area.h - 64.0),
                    &rows,
                    &mut self.facility_page,
                );
                if ux::button(
                    Rect::new(area.x, area.y + area.h - 48.0, area.w, 48.0),
                    if disciples.iter().any(|d| d.rank == DiscipleRank::Outer) {
                        "Raise grade / 50 stones"
                    } else {
                        "Recruit an outer disciple to upgrade"
                    },
                    true,
                    stones >= 50 && disciples.iter().any(|d| d.rank == DiscipleRank::Outer),
                ) {
                    return Some(
                        UpdateResult::new()
                            .with_action(Action::UpgradeBuilding(building.building_type.clone())),
                    );
                }
            }
        }
        None
    }
    fn draw_plots(
        &mut self,
        area: Rect,
        building: &Building,
        data: &GameData,
    ) -> Option<UpdateResult> {
        let count = building.herb_plots.len().min(building.get_max_herb_plots());
        let range = ux::page(area, count, 88.0, &mut self.facility_page);
        for (row, index) in range.enumerate() {
            let y = area.y + row as f32 * 88.0;
            if let Some(growing) = &building.herb_plots[index].growing {
                let name = data
                    .herbs
                    .get(&growing.herb_id)
                    .map(|h| h.name.as_str())
                    .unwrap_or(&growing.herb_id);
                draw_ui_text(
                    &format!("Plot {} / {}", index + 1, name),
                    area.x + 12.0,
                    y + 28.0,
                    FONT_BODY_SIZE,
                    PRIMARY,
                );
                let status = if growing.is_mature() {
                    if building.assigned_disciple.is_some() {
                        "Ripe / Attendant harvesting".to_owned()
                    } else {
                        "Ripe / Appoint an attendant to harvest".to_owned()
                    }
                } else {
                    format!(
                        "Growing / {} remaining",
                        crate::data::time::days_label(growing.ticks_remaining)
                    )
                };
                draw_ui_text(
                    &status,
                    area.x + 12.0,
                    y + 60.0,
                    FONT_BODY_SIZE,
                    TEXT_SECONDARY,
                );
            } else if ux::button(
                Rect::new(area.x, y, area.w, 56.0),
                &format!("Plot {} / Choose a seed", index + 1),
                true,
                true,
            ) {
                self.herb_planting_modal = Some(index);
                self.facility_page = 0;
            }
        }
        if count == 0 {
            draw_ui_text(
                "No plots are available at this grade.",
                area.x,
                area.y + 30.0,
                FONT_BODY_SIZE,
                TEXT_SECONDARY,
            );
        }
        None
    }
    fn draw_seed_selection(
        &mut self,
        building: &Building,
        data: &GameData,
        season: &Season,
        plot: usize,
    ) -> Option<UpdateResult> {
        if ux::header_to(
            "Choose a Seed",
            &format!(
                "Plot {} / Season {} / Maximum tier {}",
                plot + 1,
                season,
                building.get_max_herb_tier()
            ),
            "Back to garden",
        ) {
            self.herb_planting_modal = None;
            self.facility_page = 0;
            return None;
        }
        let area = ux::content();
        let mut herbs: Vec<_> = data
            .herbs
            .values()
            .filter(|h| h.tier <= building.get_max_herb_tier())
            .collect();
        herbs.sort_by(|a, b| {
            let available = |herb: &crate::data::herbs::Herb| {
                herb.grow_seasons.contains(season)
                    || (building.building_type == BuildingType::Greenhouse
                        && building.infused_element.as_ref() == Some(&herb.element))
            };
            available(b)
                .cmp(&available(a))
                .then(a.tier.cmp(&b.tier))
                .then(a.name.cmp(&b.name))
        });
        let range = ux::page(area, herbs.len(), 116.0, &mut self.facility_page);
        for (row, index) in range.enumerate() {
            let herb = herbs[index];
            let y = area.y + row as f32 * 116.0;
            let available = herb.grow_seasons.contains(season)
                || (building.building_type == BuildingType::Greenhouse
                    && building.infused_element.as_ref() == Some(&herb.element));
            if ux::button(
                Rect::new(area.x, y, area.w, 46.0),
                &format!("Sow {} / {} / Tier {}", herb.name, herb.element, herb.tier),
                true,
                available,
            ) {
                self.herb_planting_modal = None;
                self.facility_page = 0;
                return Some(UpdateResult::new().with_action(Action::PlantHerb(
                    building.id,
                    plot,
                    herb.id.clone(),
                )));
            }
            let seasons = herb
                .grow_seasons
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            draw_ui_text(
                &format!(
                    "{} / Seasons: {}{}",
                    crate::data::time::days_label(herb.grow_time_ticks),
                    seasons,
                    if available { "" } else { " / Out of season" }
                ),
                area.x + 12.0,
                y + 72.0,
                FONT_SMALL_SIZE,
                if available { TEXT_SECONDARY } else { WARNING },
            );
            ux::clipped(Rect::new(area.x, y + 78.0, area.w, 30.0), || {
                draw_wrapped_text(
                    &herb.description,
                    area.x + 12.0,
                    y + 98.0,
                    area.w - 24.0,
                    FONT_SMALL_SIZE,
                    TEXT_SECONDARY,
                );
            });
        }
        None
    }
    pub(super) fn draw_attendants(
        &mut self,
        area: Rect,
        building: &Building,
        data: &GameData,
        disciples: &[Disciple],
    ) -> Option<UpdateResult> {
        let current = building
            .assigned_disciple
            .and_then(|id| disciples.iter().find(|d| d.id == id))
            .map(|d| d.name.as_str())
            .unwrap_or("None");
        draw_ui_text(
            &format!("Current attendant: {}", current),
            area.x,
            area.y + 26.0,
            FONT_BODY_SIZE,
            PRIMARY,
        );
        if ux::button(
            Rect::new(area.x + area.w - 164.0, area.y, 164.0, 44.0),
            "Recall attendant",
            false,
            building.assigned_disciple.is_some(),
        ) {
            return Some(
                UpdateResult::new()
                    .with_action(Action::AssignDiscipleToBuilding(building.id, None)),
            );
        }
        let list = Rect::new(area.x, area.y + 60.0, area.w, area.h - 60.0);
        let choices: Vec<_> = disciples
            .iter()
            .filter(|d| d.rank == DiscipleRank::Outer)
            .collect();
        let range = ux::page(list, choices.len(), 60.0, &mut self.facility_page);
        for (row, index) in range.enumerate() {
            let disciple = choices[index];
            let elsewhere = data
                .buildings
                .iter()
                .any(|b| b.id != building.id && b.assigned_disciple == Some(disciple.id));
            if ux::button(
                Rect::new(list.x, list.y + row as f32 * 60.0, list.w, 48.0),
                &format!(
                    "{} / Spirit {} / Mind {}{}",
                    disciple.name,
                    disciple.attributes.spirit,
                    disciple.attributes.mind,
                    if elsewhere {
                        " / Serving another hall"
                    } else {
                        ""
                    }
                ),
                building.assigned_disciple == Some(disciple.id),
                !elsewhere,
            ) {
                return Some(
                    UpdateResult::new().with_action(Action::AssignDiscipleToBuilding(
                        building.id,
                        Some(disciple.id),
                    )),
                );
            }
        }
        if choices.is_empty() {
            draw_ui_text(
                "Recruit outer disciples at the Sect Hall to staff this hall.",
                list.x,
                list.y + 30.0,
                FONT_BODY_SIZE,
                TEXT_SECONDARY,
            );
        }
        None
    }
    fn draw_stores(
        &mut self,
        area: Rect,
        building: &Building,
        data: &GameData,
        inventory: &HashMap<String, u32>,
    ) -> Option<UpdateResult> {
        let drying = building.building_type == BuildingType::DryingPavilion;
        let mut herbs: Vec<_> = data
            .herbs
            .values()
            .filter(|herb| {
                !herb.id.starts_with("dried_")
                    && (inventory.get(&herb.id).copied().unwrap_or(0) > 0
                        || inventory
                            .get(&format!("dried_{}", herb.id))
                            .copied()
                            .unwrap_or(0)
                            > 0)
            })
            .collect();
        herbs.sort_by(|a, b| a.name.cmp(&b.name));
        let output = ((5.0 * (1.0 - building.get_drying_loss_rate())).ceil() as u32).max(1);
        let description = if drying {
            format!("Preserve 5 fresh herbs into {} dried herbs.", output)
        } else {
            format!(
                "Preservation seal reduces decay by {:.0}%.",
                building.get_decay_reduction() * 100.0
            )
        };
        draw_ui_text(&description, area.x, area.y + 26.0, FONT_BODY_SIZE, PRIMARY);
        let list = Rect::new(area.x, area.y + 48.0, area.w, area.h - 48.0);
        let range = ux::page(list, herbs.len(), 64.0, &mut self.facility_page);
        for (row, index) in range.enumerate() {
            let herb = herbs[index];
            let fresh = inventory.get(&herb.id).copied().unwrap_or(0);
            let dried = inventory
                .get(&format!("dried_{}", herb.id))
                .copied()
                .unwrap_or(0);
            let label = format!(
                "{} / {} fresh / {} dried{}",
                herb.name,
                fresh,
                dried,
                if drying { " / Preserve 5" } else { "" }
            );
            let row_rect = Rect::new(list.x, list.y + row as f32 * 64.0, list.w, 48.0);
            if drying {
                if ux::button(row_rect, &label, true, fresh >= 5) {
                    return Some(
                        UpdateResult::new().with_action(Action::ProcessDryingPavilion(
                            building.id,
                            herb.id.clone(),
                        )),
                    );
                }
            } else {
                draw_ui_text(
                    &label,
                    row_rect.x + 8.0,
                    row_rect.y + 28.0,
                    FONT_BODY_SIZE,
                    TEXT_PRIMARY,
                );
            }
        }
        if herbs.is_empty() {
            draw_ui_text(
                "No herbs are stored yet.",
                list.x,
                list.y + 30.0,
                FONT_BODY_SIZE,
                TEXT_SECONDARY,
            );
        }
        None
    }
}
