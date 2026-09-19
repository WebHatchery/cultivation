//! Paged reference material, separated from memorials and live research actions.
use crate::data::{history::DeceasedDisciple, loader::GameData};
use crate::state::{StateTransition, UpdateResult};
use crate::ui::workspace as ux;

pub struct LibraryState {
    tab: usize,
    page: usize,
}
impl LibraryState {
    pub fn new() -> Self {
        Self { tab: 0, page: 0 }
    }
    pub fn update(
        &mut self,
        data: &GameData,
        spirit_stones: u32,
        deceased: &[DeceasedDisciple],
    ) -> UpdateResult {
        if ux::header(
            "Ancestral Archive",
            "Read teachings and remember fallen disciples. Recover doctrine at the Sect Hall.",
        ) {
            return UpdateResult::new().with_transition(StateTransition::ToSectBase);
        }
        let mut area = ux::content();
        if ux::tabs(
            area,
            &["Cultivation laws", "Doctrine reference", "Memorials"],
            &mut self.tab,
        ) {
            self.page = 0;
        }
        area.y += 60.0;
        area.h -= 60.0;
        let mut rows = Vec::new();
        match self.tab {
            0 => {
                let mut laws: Vec<_> = data.laws.values().collect();
                laws.sort_by(|a, b| a.name.cmp(&b.name));
                rows.extend(
                    laws.iter()
                        .map(|law| format!("{} / {} — {}", law.name, law.element, law.description)),
                );
            }
            1 => {
                let mut techs: Vec<_> = data.techs.values().collect();
                techs.sort_by(|a, b| {
                    a.cost_spirit_stones
                        .cmp(&b.cost_spirit_stones)
                        .then(a.name.cmp(&b.name))
                });
                rows.push(format!("Treasury: {} stones. This is the full reference, including unrecovered teachings.", spirit_stones));
                rows.extend(techs.iter().map(|tech| {
                    format!(
                        "{} / {} stones — {}",
                        tech.name, tech.cost_spirit_stones, tech.description
                    )
                }));
            }
            _ => {
                rows.extend(deceased.iter().rev().map(|disciple| {
                    format!(
                        "{} / {} — {} (tick {})",
                        disciple.name,
                        disciple.realm_at_death.replace('_', " "),
                        disciple.cause_of_death,
                        disciple.tick_of_death
                    )
                }));
                if rows.is_empty() {
                    rows.push("No disciple has fallen. Prepare halls, training, and medicine to keep these tablets empty.".to_owned());
                }
            }
        }
        ux::text_pages(area, &rows, &mut self.page);
        UpdateResult::new()
    }
    pub fn draw(&self, _data: &GameData, _spirit_stones: u32, _deceased: &[DeceasedDisciple]) {}
}
