//! Market intelligence grouped by the question the player is answering.
use crate::engine::world_sim::WorldSim;
use crate::state::{StateTransition, UpdateResult};
use crate::ui::{components::*, theme::*, workspace as ux};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;
use std::collections::HashMap;

pub struct TradeScreenState {
    selected_node: Option<usize>,
    tab: usize,
    market_page: usize,
    detail_page: usize,
}
impl TradeScreenState {
    pub fn new() -> Self {
        Self {
            selected_node: None,
            tab: 0,
            market_page: 0,
            detail_page: 0,
        }
    }
    pub fn update(
        &mut self,
        world_sim: &WorldSim,
        spirit_stones: u32,
        inventory: &HashMap<String, u32>,
    ) -> UpdateResult {
        if ux::header(
            "Caravan Ledger",
            &format!(
                "{} spirit stones / Inspect prices and road conditions before planning a journey.",
                spirit_stones
            ),
        ) {
            return UpdateResult::new().with_transition(StateTransition::ToSectBase);
        }
        let mut area = ux::content();
        if ux::tabs(
            area,
            &["Markets", "Route safety", "Sect stores"],
            &mut self.tab,
        ) {
            self.detail_page = 0;
        }
        area.y += 60.0;
        area.h -= 60.0;
        match self.tab {
            0 => self.draw_markets(area, world_sim),
            1 => {
                let rows: Vec<String> = world_sim
                    .economy
                    .routes
                    .iter()
                    .map(|route| {
                        format!(
                            "{} to {} — {} / {}",
                            node_name(world_sim, &route.from_node),
                            node_name(world_sim, &route.to_node),
                            if route.active {
                                safety_label(route.safety_rating)
                            } else {
                                "Closed"
                            },
                            crate::data::time::days_label(route.travel_ticks)
                        )
                    })
                    .collect();
                ux::text_pages(area, &rows, &mut self.detail_page);
            }
            _ => {
                let mut goods: Vec<_> = inventory.iter().filter(|(_, count)| **count > 0).collect();
                goods.sort_by_key(|(id, _)| item_name(id));
                let mut rows = vec![format!(
                    "Seasonal caravan activity: {:.0}%",
                    world_sim.seasonal_modifiers.trade_activity_mod * 100.0
                )];
                rows.extend(
                    goods
                        .iter()
                        .map(|(id, count)| format!("{}: {}", item_name(id), count)),
                );
                if goods.is_empty() {
                    rows.push("No stored trade goods.".to_owned());
                }
                ux::text_pages(area, &rows, &mut self.detail_page);
            }
        }
        UpdateResult::new()
    }
    fn draw_markets(&mut self, area: Rect, world_sim: &WorldSim) {
        let (list, detail) = ux::split(area);
        let nodes = &world_sim.economy.nodes;
        if self.selected_node.is_none() && !nodes.is_empty() {
            self.selected_node = Some(0);
        }
        let range = ux::page(list, nodes.len(), 72.0, &mut self.market_page);
        for (row, index) in range.enumerate() {
            let node = &nodes[index];
            let y = list.y + row as f32 * 72.0;
            if ux::button(
                Rect::new(list.x, y, list.w, 44.0),
                &node.name,
                self.selected_node == Some(index),
                true,
            ) {
                self.selected_node = Some(index);
                self.detail_page = 0;
            }
            draw_ui_text(
                safety_label(average_route_safety(world_sim, &node.id)),
                list.x + 12.0,
                y + 64.0,
                FONT_SMALL_SIZE,
                SECONDARY,
            );
        }
        draw_panel(detail, Some("Market report"));
        if let Some(node) = self.selected_node.and_then(|index| nodes.get(index)) {
            let mut goods: Vec<_> = node.base_prices.iter().collect();
            goods.sort_by_key(|(id, _)| item_name(id));
            let mut rows = vec![format!(
                "{} / {}",
                node.name,
                controller_label(world_sim, &node.controller_faction_id)
            )];
            rows.extend(goods.iter().map(|(id, base)| {
                format!(
                    "{}: {} stones / Supply {} / Demand {:.1}x",
                    item_name(id),
                    world_sim.get_item_price(&node.id, id).unwrap_or(**base),
                    node.supply.get(*id).copied().unwrap_or(0),
                    node.demand.get(*id).copied().unwrap_or(1.0)
                )
            }));
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
    }
    pub fn draw(&self, _world_sim: &WorldSim) {}
}

fn controller_label(world_sim: &WorldSim, controller_id: &str) -> String {
    if controller_id.is_empty() {
        "Independent caravan court".to_string()
    } else {
        world_sim
            .get_faction(controller_id)
            .map(|faction| faction.name.clone())
            .unwrap_or_else(|| controller_id.replace('_', " "))
    }
}

fn average_route_safety(world_sim: &WorldSim, node_id: &str) -> f32 {
    let mut total = 0.0;
    let mut count = 0.0;
    for route in &world_sim.economy.routes {
        if route.from_node == node_id || route.to_node == node_id {
            total += if route.active {
                route.safety_rating
            } else {
                0.0
            };
            count += 1.0;
        }
    }
    if count == 0.0 {
        1.0
    } else {
        total / count
    }
}

fn safety_label(safety: f32) -> &'static str {
    if safety >= 0.75 {
        "Auspicious"
    } else if safety >= 0.45 {
        "Risky"
    } else {
        "Disrupted"
    }
}

fn node_name(world_sim: &WorldSim, node_id: &str) -> String {
    world_sim
        .economy
        .get_node(node_id)
        .map(|node| node.name.clone())
        .unwrap_or_else(|| node_id.replace('_', " "))
}

fn item_name(item_id: &str) -> String {
    item_id
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
