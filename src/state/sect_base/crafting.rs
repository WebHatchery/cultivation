use super::*;
use macroquad_toolkit::ui::draw_ui_text;

impl SectBaseState {
    pub(super) fn draw_construction_modal(
        &mut self,
        data: &GameData,
        unlocked_techs: &[String],
        spirit_stones: u32,
    ) -> Option<UpdateResult> {
        let (screen_w, screen_h) = (screen_width(), screen_height());
        draw_rectangle(
            0.0,
            0.0,
            screen_w,
            screen_h,
            Color::new(0.0, 0.0, 0.0, 0.76),
        );

        let modal_w = 620.0_f32.min(screen_w - 60.0);
        let modal_h = 560.0_f32.min(screen_h - 70.0);
        let modal_x = (screen_w - modal_w) / 2.0;
        let modal_y = (screen_h - modal_h) / 2.0;

        draw_panel(
            Rect::new(modal_x, modal_y, modal_w, modal_h),
            Some("Hall-Raising Ledger"),
        );

        if draw_button(
            Rect::new(modal_x + modal_w - 76.0, modal_y + 10.0, 64.0, 30.0),
            "Back",
            false,
        ) {
            self.crafting_modal_open = false;
            return None;
        }

        draw_construction_intro(
            Rect::new(modal_x + 22.0, modal_y + 54.0, modal_w - 44.0, 86.0),
            spirit_stones,
        );

        let content_y = modal_y + 152.0;
        let content_h = modal_h - 216.0;
        let content_rect = Rect::new(modal_x + 18.0, content_y, modal_w - 36.0, content_h);

        let mut build_opts: Vec<_> = data.building_definitions.values().collect();
        build_opts.sort_by_key(|a| a.cost);

        let available_defs: Vec<_> = build_opts
            .into_iter()
            .filter(|def| {
                let req_tech = def.tech_required.clone().unwrap_or_default();
                let tech_unlocked = unlocked_techs.contains(&req_tech) || req_tech.is_empty();
                let already_built = def.unique
                    && data
                        .buildings
                        .iter()
                        .any(|b| b.building_type == def.building_type);
                tech_unlocked && !already_built
            })
            .collect();

        let item_height = 108.0;
        let per_page = (content_h / item_height).floor().max(1.0) as usize;
        let pages = available_defs.len().div_ceil(per_page).max(1);
        let mut page = (self.construction_scroll as usize).min(pages - 1);
        if content_rect.contains(mouse_position().into()) {
            if mouse_wheel().1 < 0.0 {
                page = (page + 1).min(pages - 1);
            }
            if mouse_wheel().1 > 0.0 {
                page = page.saturating_sub(1);
            }
        }
        let footer_y = modal_y + modal_h - 52.0;
        if page > 0
            && draw_button(
                Rect::new(modal_x + 20.0, footer_y, 100.0, 44.0),
                "Previous",
                false,
            )
        {
            page -= 1;
        }
        if page + 1 < pages
            && draw_button(
                Rect::new(modal_x + modal_w - 120.0, footer_y, 100.0, 44.0),
                "Next",
                false,
            )
        {
            page += 1;
        }
        self.construction_scroll = page as f32;
        draw_ui_text(
            &format!("{} / {}", page + 1, pages),
            modal_x + modal_w * 0.5 - 20.0,
            footer_y + 28.0,
            FONT_SMALL_SIZE,
            TEXT_SECONDARY,
        );

        if available_defs.is_empty() {
            draw_wrapped_text(
                "No new hall plans are ready. Recover more doctrine in the archive or research the missing techniques before expanding the mountain.",
                content_rect.x + 18.0,
                content_rect.y + 34.0,
                content_rect.w - 36.0,
                FONT_BODY_SIZE,
                TEXT_SECONDARY,
            );
            return None;
        }

        let mut b_y = content_y;
        for def in available_defs
            .into_iter()
            .skip(page * per_page)
            .take(per_page)
        {
            let card = Rect::new(
                content_rect.x,
                b_y,
                content_rect.w - 10.0,
                item_height - 12.0,
            );
            let affordable = spirit_stones >= def.cost;
            if def.building_type == BuildingType::MissionBoard {
                self.register_tutorial_target(2, card);
            }
            if draw_building_plan_card(card, def, affordable) && affordable {
                self.crafting_modal_open = false;
                self.placement_mode = Some(def.building_type.clone());
            }

            b_y += item_height;
        }

        None
    }

    pub(super) fn draw_crafting_modal(
        &mut self,
        data: &GameData,
        b_type: &BuildingType,
        spirit_stones: u32,
        herbs: u32,
        inventory: &std::collections::HashMap<String, u32>,
        disciples: &[Disciple],
        discovered_recipes: &[String],
    ) -> Option<UpdateResult> {
        use crate::engine::crafting::{self, CraftingContext};

        let (screen_w, screen_h) = (screen_width(), screen_height());
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.8));

        let modal_w = 550.0;
        let modal_h = 650.0;
        let modal_x = (screen_w - modal_w) / 2.0;
        let modal_y = (screen_h - modal_h) / 2.0;

        draw_panel(
            Rect::new(modal_x, modal_y, modal_w, modal_h),
            Some("Sect Workshop Ledger"),
        );

        if draw_button(
            Rect::new(modal_x + modal_w - 60.0, modal_y + 10.0, 50.0, 30.0),
            "Seal",
            false,
        ) {
            self.crafting_modal_open = false;
        }

        let building = data
            .buildings
            .iter()
            .find(|b| b.building_type == *b_type && b.status == BuildingStatus::Active);
        let building_level = building.map(|b| b.level).unwrap_or(1);
        let building_element = building.map(|b| b.element.clone()).unwrap_or_default();

        let assigned_disciple = building
            .and_then(|b| b.assigned_disciple)
            .and_then(|did| disciples.iter().find(|d| d.id == did));
        let disciple_mind = assigned_disciple.map(|d| d.attributes.mind);

        let info_y = modal_y + 45.0;
        draw_ui_text(
            &format!("Hall grade {}", building_level),
            modal_x + 20.0,
            info_y,
            FONT_SMALL_SIZE,
            TEXT_SECONDARY,
        );
        if let Some(d) = assigned_disciple {
            draw_ui_text(
                &format!("Attendant: {} (Mind Sea: {})", d.name, d.attributes.mind),
                modal_x + 150.0,
                info_y,
                FONT_SMALL_SIZE,
                TEXT_SECONDARY,
            );
        } else {
            draw_ui_text(
                "No attendant appointed",
                modal_x + 150.0,
                info_y,
                FONT_SMALL_SIZE,
                TEXT_SECONDARY,
            );
        }

        let mut r_y = modal_y + 70.0;
        let recipes: Vec<_> = data
            .recipes
            .iter()
            .filter(|r| r.required_building == *b_type && discovered_recipes.contains(&r.id))
            .collect();

        if recipes.is_empty() {
            draw_ui_text(
                "No recovered formulas for this workshop.",
                modal_x + 20.0,
                r_y,
                FONT_BODY_SIZE,
                TEXT_SECONDARY,
            );
            return None;
        }

        let ctx = CraftingContext {
            building_level,
            building_element,
            disciple_mind,
        };

        for recipe in recipes {
            if r_y > modal_y + modal_h - 30.0 {
                draw_ui_text("...", modal_x + 20.0, r_y, FONT_SMALL_SIZE, TEXT_SECONDARY);
                break;
            }

            if let Some(res) = self.draw_recipe_entry(
                recipe,
                &ctx,
                modal_x,
                modal_w,
                &mut r_y,
                modal_y + modal_h,
                spirit_stones,
                herbs,
                inventory,
            ) {
                return Some(res);
            }
        }

        None
    }

    fn draw_recipe_entry(
        &self,
        recipe: &crate::data::items::Recipe,
        ctx: &crate::engine::crafting::CraftingContext,
        modal_x: f32,
        modal_w: f32,
        r_y: &mut f32,
        _max_y: f32,
        spirit_stones: u32,
        herbs: u32,
        inventory: &std::collections::HashMap<String, u32>,
    ) -> Option<UpdateResult> {
        use crate::engine::crafting;

        let success_chance = crafting::calculate_success_chance(recipe, ctx);

        let mut can_craft = true;
        let mut ing_parts: Vec<String> = Vec::new();
        for (ing, amt) in recipe.ingredients.iter() {
            let ing: &String = ing;
            let has: u32 = match ing.as_str() {
                "spirit_stones" => spirit_stones,
                "herbs" => herbs,
                _ => *inventory.get(ing).unwrap_or(&0),
            };
            if has < *amt {
                can_craft = false;
            }
            ing_parts.push(format!("{}x {} ({}/{})", amt, ing, has, amt));
        }

        let chance_color = if success_chance > 70 {
            Color::new(0.2, 0.8, 0.2, 1.0)
        } else if success_chance > 40 {
            Color::new(0.9, 0.8, 0.2, 1.0)
        } else {
            Color::new(0.9, 0.3, 0.2, 1.0)
        };

        let label = format!("{} [{}%]", recipe.name, success_chance);

        if draw_button(
            Rect::new(modal_x + 20.0, *r_y, modal_w - 40.0, 35.0),
            &label,
            false,
        ) {
            if can_craft {
                return Some(UpdateResult::new().with_action(Action::CraftItem(recipe.id.clone())));
            }
        }

        draw_ui_text(
            &format!("{}%", success_chance),
            modal_x + modal_w - 70.0,
            *r_y + 22.0,
            FONT_SMALL_SIZE,
            chance_color,
        );

        let ing_text = ing_parts.join(", ");
        let status_text = if can_craft {
            ing_text
        } else {
            format!("LACKING: {}", ing_text)
        };
        let status_color = if can_craft {
            TEXT_SECONDARY
        } else {
            Color::new(0.8, 0.3, 0.3, 1.0)
        };
        draw_ui_text(
            &status_text,
            modal_x + 25.0,
            *r_y + 50.0,
            FONT_SMALL_SIZE,
            status_color,
        );

        *r_y += 65.0;

        None
    }
}

fn draw_construction_intro(rect: Rect, spirit_stones: u32) {
    draw_ui_text(
        "Choose a hall, then tap a clear spot on the map.",
        rect.x,
        rect.y + 28.0,
        FONT_BODY_SIZE,
        TEXT_PRIMARY,
    );
    draw_ui_text(
        &format!("Treasury: {} spirit stones", spirit_stones),
        rect.x,
        rect.y + 58.0,
        FONT_BODY_SIZE,
        TEXT_HIGHLIGHT,
    );
}

fn draw_building_plan_card(
    rect: Rect,
    def: &crate::data::loader::BuildingDefinition,
    affordable: bool,
) -> bool {
    let hovered = rect.contains(mouse_position().into());
    let accent = building_plan_color(&def.building_type);
    let alpha = if hovered { 0.54 } else { 0.38 };

    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.035, 0.026, 0.018, alpha),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if hovered { 2.0 } else { 1.0 },
        Color::new(
            accent.r,
            accent.g,
            accent.b,
            if affordable { 0.62 } else { 0.28 },
        ),
    );
    draw_rectangle(
        rect.x,
        rect.y,
        5.0,
        rect.h,
        Color::new(
            accent.r,
            accent.g,
            accent.b,
            if affordable { 0.76 } else { 0.34 },
        ),
    );

    draw_ui_text(
        &def.name,
        rect.x + 18.0,
        rect.y + 28.0,
        FONT_BODY_SIZE,
        if affordable {
            TEXT_PRIMARY
        } else {
            TEXT_SECONDARY
        },
    );
    draw_ui_text(
        building_plan_role(&def.building_type),
        rect.x + 18.0,
        rect.y + 52.0,
        FONT_SMALL_SIZE,
        accent,
    );
    draw_wrapped_text(
        &def.description,
        rect.x + 18.0,
        rect.y + 76.0,
        rect.w - 160.0,
        FONT_SMALL_SIZE,
        TEXT_SECONDARY,
    );

    let cost_text = format!("{} SS", def.cost);
    draw_ui_text(
        &cost_text,
        rect.x + rect.w - 118.0,
        rect.y + 32.0,
        FONT_HEADER_SIZE,
        if affordable { TEXT_HIGHLIGHT } else { FAILURE },
    );
    draw_ui_text(
        if affordable {
            "Place foundation"
        } else {
            "Need more stones"
        },
        rect.x + rect.w - 126.0,
        rect.y + 66.0,
        FONT_SMALL_SIZE,
        if affordable { TEXT_SECONDARY } else { FAILURE },
    );
    draw_ui_text(
        &format!("Aspect: {}", def.element),
        rect.x + rect.w - 126.0,
        rect.y + 88.0,
        FONT_SMALL_SIZE,
        TEXT_SECONDARY,
    );

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn building_plan_role(building_type: &BuildingType) -> &'static str {
    match building_type {
        BuildingType::SectHall => "Ancestral command",
        BuildingType::Dormitory => "Disciple shelter",
        BuildingType::TrainingYard => "Cultivation ascent",
        BuildingType::LibraryPavilion => "Memory and scripture",
        BuildingType::MissionBoard => "Beyond-gate dispatch",
        BuildingType::SpiritGarden => "Ambient Qi income",
        BuildingType::Decoration => "Mountain dignity",
        BuildingType::AlchemyFurnace => "Pill refinement",
        BuildingType::ArtifactForge => "Artifact forging",
        BuildingType::Blacksmith => "Weapons and armor",
        BuildingType::HerbGarden => "Medicine roots",
        BuildingType::Greenhouse => "Rare herb sanctuary",
        BuildingType::DryingPavilion => "Herb preservation",
        BuildingType::HerbStorage => "Winter stores",
        BuildingType::TalismanScriptorium => "Talismans and seals",
    }
}

fn building_plan_color(building_type: &BuildingType) -> Color {
    match building_type {
        BuildingType::SectHall | BuildingType::Dormitory | BuildingType::HerbStorage => PRIMARY,
        BuildingType::TrainingYard | BuildingType::Blacksmith | BuildingType::ArtifactForge => {
            WARNING
        }
        BuildingType::LibraryPavilion | BuildingType::TalismanScriptorium => SECONDARY,
        BuildingType::MissionBoard => ACCENT,
        BuildingType::SpiritGarden
        | BuildingType::HerbGarden
        | BuildingType::Greenhouse
        | BuildingType::DryingPavilion => SUCCESS,
        BuildingType::AlchemyFurnace => ACCENT,
        BuildingType::Decoration => TEXT_HIGHLIGHT,
    }
}
