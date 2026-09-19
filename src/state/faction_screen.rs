//! Rival selection with relationship-first records; no inactive diplomacy controls.
use crate::data::factions::{Faction, FactionSpecialty, FactionType};
use crate::data::relations::ReputationTier;
use crate::engine::world_sim::WorldSim;
use crate::state::{StateTransition, UpdateResult};
use crate::ui::{components::*, theme::*, workspace as ux};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub struct FactionScreenState {
    selected_faction: Option<usize>,
    page: usize,
    detail_page: usize,
}
impl FactionScreenState {
    pub fn new() -> Self {
        Self {
            selected_faction: None,
            page: 0,
            detail_page: 0,
        }
    }
    pub fn update(&mut self, world_sim: &WorldSim) -> UpdateResult {
        if ux::header(
            "Regional Powers",
            "Select a power to inspect its standing, strength, and specialty.",
        ) {
            return UpdateResult::new().with_transition(StateTransition::ToSectBase);
        }
        let (list, detail) = ux::split(ux::content());
        if self.selected_faction.is_none() && !world_sim.factions.is_empty() {
            self.selected_faction = Some(0);
        }
        let range = ux::page(list, world_sim.factions.len(), 76.0, &mut self.page);
        for (row, index) in range.enumerate() {
            let faction = &world_sim.factions[index];
            let y = list.y + row as f32 * 76.0;
            if ux::button(
                Rect::new(list.x, y, list.w, 46.0),
                &faction.name,
                self.selected_faction == Some(index),
                true,
            ) {
                self.selected_faction = Some(index);
                self.detail_page = 0;
            }
            let tier = world_sim
                .get_relation(&faction.id)
                .map(|r| r.reputation_tier())
                .unwrap_or(ReputationTier::Neutral);
            draw_ui_text(
                &tier.to_string(),
                list.x + 12.0,
                y + 66.0,
                FONT_SMALL_SIZE,
                tier_color(tier),
            );
        }
        draw_panel(detail, Some("Power record"));
        if let Some(faction) = self
            .selected_faction
            .and_then(|i| world_sim.factions.get(i))
        {
            let relation = world_sim.get_relation(&faction.id);
            let standing = relation
                .map(|r| {
                    if r.at_war {
                        "At war with your sect".to_owned()
                    } else if let Some(treaty) = &r.treaty {
                        format!("Treaty: {}", treaty.name())
                    } else {
                        format!("Standing: {} ({})", r.reputation, r.reputation_tier())
                    }
                })
                .unwrap_or_else(|| "Standing unknown".to_owned());
            let rows = vec![
                format!(
                    "{} / {}",
                    faction.name,
                    faction_type_title(&faction.faction_type)
                ),
                standing,
                format!(
                    "Regional power: {} / Treasury: {} stones",
                    faction.power_level, faction.wealth
                ),
                format!(
                    "Leader: {} / {}",
                    leader_title(faction),
                    realm_label(&faction.leader_realm)
                ),
                format!("Specialty: {}", specialty_label(&faction.specialty)),
                format!("Signature art: {}", signature_technique(faction)),
                faction.description.clone(),
            ];
            ux::text_pages(
                Rect::new(
                    detail.x + 20.0,
                    detail.y + 56.0,
                    detail.w - 40.0,
                    detail.h - 72.0,
                ),
                &rows,
                &mut self.detail_page,
            );
        }
        UpdateResult::new()
    }
    pub fn draw(&self, _world_sim: &WorldSim) {}
}

fn faction_type_title(faction_type: &FactionType) -> &'static str {
    match faction_type {
        FactionType::Sect => "Orthodox Cultivation Sect",
        FactionType::MerchantGuild => "Merchant Cultivator Guild",
        FactionType::DemonCult => "Forbidden Moon Cult",
        FactionType::ImperialCourt => "Imperial Cultivation Court",
        FactionType::BanditClan => "Rogue Mountain Clan",
        FactionType::BeastHorde => "Spirit Beast Horde",
    }
}

fn specialty_label(specialty: &FactionSpecialty) -> &'static str {
    match specialty {
        FactionSpecialty::Trade => "Caravan and auction houses",
        FactionSpecialty::Combat => "Sword and body cultivation",
        FactionSpecialty::Alchemy => "Pill refining",
        FactionSpecialty::Formations => "Arrays and seals",
        FactionSpecialty::Intelligence => "Secrets and shadows",
        FactionSpecialty::Agriculture => "Spirit herb gardens",
    }
}

fn signature_technique(faction: &Faction) -> &'static str {
    match faction.specialty {
        FactionSpecialty::Trade => "Golden Scale Exchange Art",
        FactionSpecialty::Combat => "Crimson Mountain Breaking Fist",
        FactionSpecialty::Alchemy => "Azure Cauldron Returning Breath",
        FactionSpecialty::Formations => "Nine Palace Imperial Seal",
        FactionSpecialty::Intelligence => "Shadow Moon Veil",
        FactionSpecialty::Agriculture => "Jade Lotus Verdant Method",
    }
}

fn leader_title(faction: &Faction) -> String {
    let title = match faction.faction_type {
        FactionType::Sect => "Elder",
        FactionType::MerchantGuild => "Guildmaster",
        FactionType::DemonCult => "Moon Hierarch",
        FactionType::ImperialCourt => "Imperial Envoy",
        FactionType::BanditClan => "Mountain Chief",
        FactionType::BeastHorde => "Beast Sovereign",
    };
    format!("{} {}", title, leader_name_seed(&faction.name))
}

fn leader_name_seed(name: &str) -> &'static str {
    if name.contains("Azure") {
        "Blue Crane"
    } else if name.contains("Golden") {
        "Gold Abacus"
    } else if name.contains("Crimson") {
        "Red Fang"
    } else if name.contains("Shadow") {
        "Silent Moon"
    } else if name.contains("Imperial") {
        "Iron Seal"
    } else if name.contains("Lotus") {
        "Jade Rain"
    } else {
        "Broken Blade"
    }
}

fn realm_label(id: &str) -> String {
    if id.is_empty() {
        return "Unknown".to_string();
    }
    id.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn tier_color(tier: ReputationTier) -> Color {
    let (r, g, b) = tier.color();
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}
