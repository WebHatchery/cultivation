//! Focused doctrine browser: filter, select, then review a single recovery.
use crate::data::loader::GameData;
use crate::data::tech::Technology;
use crate::engine::actions::Action;
use crate::ui::components::*;
use crate::ui::theme::*;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub struct TechTreeState {
    pub closed: bool,
    selected: Option<String>,
    filter: usize,
    page: usize,
}

impl TechTreeState {
    pub fn new() -> Self {
        Self {
            closed: false,
            selected: None,
            filter: 0,
            page: 0,
        }
    }
}

fn prerequisites_met(tech: &Technology, unlocked: &[String]) -> bool {
    tech.prerequisites.iter().all(|id| unlocked.contains(id))
}

pub fn draw_tech_tree_modal(
    state: &mut TechTreeState,
    data: &GameData,
    unlocked: &[String],
    stones: u32,
) -> Option<Action> {
    state.closed = false;
    let width = screen_width();
    let height = screen_height();
    draw_rectangle(0.0, 0.0, width, height, PANEL_BG);
    draw_screen_title(
        "Recover Doctrine",
        "Choose a teaching. Review what it opens before spending stones.",
        24.0,
        40.0,
    );
    if draw_button(
        Rect::new(width - 150.0, 20.0, 126.0, 44.0),
        "Back to Hall",
        false,
    ) {
        state.closed = true;
        return None;
    }
    draw_ui_text(
        &format!(
            "Treasury: {} spirit stones  /  {} teachings recovered",
            stones,
            unlocked.len()
        ),
        24.0,
        100.0,
        FONT_BODY_SIZE,
        TEXT_HIGHLIGHT,
    );
    let list_w = (width * 0.38).clamp(280.0, 420.0);
    let tabs = ["Available", "Locked", "Recovered"];
    for (index, label) in tabs.iter().enumerate() {
        let rect = Rect::new(
            24.0 + index as f32 * (list_w / 3.0),
            120.0,
            list_w / 3.0 - 4.0,
            44.0,
        );
        if draw_button(rect, label, state.filter == index) {
            state.filter = index;
            state.page = 0;
            state.selected = None;
        }
    }
    let mut techs: Vec<_> = data
        .techs
        .values()
        .filter(|tech| {
            let recovered = unlocked.contains(&tech.id);
            match state.filter {
                0 => !recovered && prerequisites_met(tech, unlocked),
                1 => !recovered && !prerequisites_met(tech, unlocked),
                _ => recovered,
            }
        })
        .collect();
    techs.sort_by(|a, b| {
        a.cost_spirit_stones
            .cmp(&b.cost_spirit_stones)
            .then(a.name.cmp(&b.name))
    });
    let per_page = ((height - 260.0) / 78.0).floor().max(1.0) as usize;
    let pages = techs.len().div_ceil(per_page).max(1);
    state.page = state.page.min(pages - 1);
    if !techs
        .iter()
        .any(|tech| Some(&tech.id) == state.selected.as_ref())
    {
        state.selected = techs.first().map(|tech| tech.id.clone());
    }
    for (row, tech) in techs
        .iter()
        .skip(state.page * per_page)
        .take(per_page)
        .enumerate()
    {
        let rect = Rect::new(24.0, 180.0 + row as f32 * 78.0, list_w - 4.0, 68.0);
        let selected = state.selected.as_ref() == Some(&tech.id);
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            if selected {
                Color::new(0.19, 0.23, 0.21, 1.0)
            } else {
                Color::new(0.09, 0.10, 0.09, 1.0)
            },
        );
        if selected {
            draw_rectangle(rect.x, rect.y, 4.0, rect.h, PRIMARY);
        }
        if draw_button(
            Rect::new(rect.x + 8.0, rect.y + 2.0, rect.w - 16.0, 38.0),
            &tech.name,
            selected,
        ) {
            state.selected = Some(tech.id.clone());
        }
        let status = if unlocked.contains(&tech.id) {
            "Recovered".to_owned()
        } else if tech.cost_spirit_stones == 0 {
            "Free recovery".to_owned()
        } else {
            format!("{} spirit stones", tech.cost_spirit_stones)
        };
        draw_ui_text(
            &status,
            rect.x + 16.0,
            rect.y + 55.0,
            FONT_SMALL_SIZE,
            TEXT_SECONDARY,
        );
    }
    if techs.is_empty() {
        draw_wrapped_text(
            "No teachings in this category. Check the other tabs for your next path.",
            32.0,
            210.0,
            list_w - 24.0,
            FONT_BODY_SIZE,
            TEXT_SECONDARY,
        );
    }
    let footer_y = height - 62.0;
    if state.page > 0 && draw_button(Rect::new(24.0, footer_y, 90.0, 44.0), "Previous", false) {
        state.page -= 1;
    }
    draw_ui_text(
        &format!("{} / {}", state.page + 1, pages),
        24.0 + list_w / 2.0 - 20.0,
        footer_y + 28.0,
        FONT_SMALL_SIZE,
        TEXT_SECONDARY,
    );
    if state.page + 1 < pages
        && draw_button(
            Rect::new(list_w - 70.0, footer_y, 90.0, 44.0),
            "Next",
            false,
        )
    {
        state.page += 1;
    }
    if let Some(tech) = state.selected.as_ref().and_then(|id| data.techs.get(id)) {
        return draw_recovery_detail(
            Rect::new(list_w + 44.0, 120.0, width - list_w - 68.0, height - 144.0),
            tech,
            data,
            unlocked,
            stones,
        );
    }
    None
}

fn draw_recovery_detail(
    rect: Rect,
    tech: &Technology,
    data: &GameData,
    unlocked: &[String],
    stones: u32,
) -> Option<Action> {
    draw_panel(rect, Some(&tech.name));
    let x = rect.x + 20.0;
    let width = rect.w - 40.0;
    let mut y = draw_wrapped_text(
        &tech.description,
        x,
        rect.y + 76.0,
        width,
        FONT_BODY_SIZE,
        TEXT_PRIMARY,
    ) + 24.0;
    let benefits: Vec<_> = tech
        .unlocks_buildings
        .iter()
        .map(|item| format!("Hall: {}", item))
        .chain(
            tech.unlocks_missions
                .iter()
                .map(|item| format!("Dispatch: {}", item)),
        )
        .collect();
    if !benefits.is_empty() {
        draw_ui_text("OPENS TO YOUR SECT", x, y, FONT_SMALL_SIZE, SECONDARY);
        y += 28.0;
        y = draw_wrapped_text(
            &benefits.join("  /  "),
            x,
            y,
            width,
            FONT_BODY_SIZE,
            TEXT_PRIMARY,
        ) + 24.0;
    }
    let missing: Vec<_> = tech
        .prerequisites
        .iter()
        .filter(|id| !unlocked.contains(id))
        .filter_map(|id| data.techs.get(id).map(|item| item.name.clone()))
        .collect();
    if !missing.is_empty() {
        draw_wrapped_text(
            &format!("Recover first: {}", missing.join(", ")),
            x,
            y,
            width,
            FONT_BODY_SIZE,
            WARNING,
        );
    }
    let footer = rect.y + rect.h - 110.0;
    draw_ink_divider(x, footer - 16.0, width);
    let recovered = unlocked.contains(&tech.id);
    let can_recover = !recovered && missing.is_empty() && stones >= tech.cost_spirit_stones;
    let status = if recovered {
        "This teaching is part of your sect's doctrine.".to_owned()
    } else if !missing.is_empty() {
        "Recover the required teaching to open this path.".to_owned()
    } else if stones < tech.cost_spirit_stones {
        format!(
            "Need {} more spirit stones.",
            tech.cost_spirit_stones - stones
        )
    } else {
        format!(
            "After recovery: {} spirit stones remain.",
            stones - tech.cost_spirit_stones
        )
    };
    draw_wrapped_text(
        &status,
        x,
        footer + 8.0,
        width,
        FONT_SMALL_SIZE,
        TEXT_SECONDARY,
    );
    if can_recover {
        let label = if tech.cost_spirit_stones == 0 {
            "Recover for free".to_owned()
        } else {
            format!("Recover / {} stones", tech.cost_spirit_stones)
        };
        if draw_button(
            Rect::new(x, rect.y + rect.h - 62.0, width, 44.0),
            &label,
            true,
        ) {
            return Some(Action::ResearchTech(tech.id.clone()));
        }
    }
    None
}
