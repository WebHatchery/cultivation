//! Readiness, history and orders occupy separate parts of a disciple record.
use super::*;
use crate::engine::actions::Action;
impl DiscipleRosterState {
    pub(super) fn draw_disciple_record(
        &mut self,
        rect: Rect,
        data: &GameData,
        disciple: &Disciple,
        index: usize,
    ) -> Option<UpdateResult> {
        draw_panel(rect, Some(&disciple.name));
        let mut body = Rect::new(rect.x + 16.0, rect.y + 54.0, rect.w - 32.0, rect.h - 70.0);
        if ux::tabs(body, &["Progress", "Lineage", "Orders"], &mut self.tab) {
            self.detail_page = 0;
        }
        body.y += 60.0;
        body.h -= 60.0;
        if self.tab == 2 {
            return self.draw_orders(body, data, disciple, index);
        }
        let stage = data.stages.get(&disciple.realm);
        let realm = stage
            .map(|stage| stage.name.as_str())
            .unwrap_or(&disciple.realm);
        let mut rows = Vec::new();
        if self.tab == 0 {
            rows.push(format!(
                "{} / Stage {} / {}",
                realm,
                disciple.sub_stage + 1,
                match disciple.rank {
                    DiscipleRank::Outer => "Outer disciple",
                    DiscipleRank::Inner => "Inner disciple",
                    DiscipleRank::Elder => "Elder",
                    DiscipleRank::SectLeader => "Patriarch",
                }
            ));
            rows.push(format!(
                "Cultivation: {} / {} experience",
                disciple.exp, disciple.exp_to_next_level
            ));
            if let Some(injury) = &disciple.injury {
                rows.push(format!(
                    "Injury: {} {}. Recovery: {} ticks remaining.",
                    injury.severity_str(),
                    injury.injury_type,
                    injury.recovery_ticks_remaining
                ));
            }
            if let Some(bottleneck) = &disciple.breakthrough_bottleneck {
                rows.push(format!(
                    "Breakthrough blocked: {}",
                    bottleneck.player_description(data)
                ));
            } else if disciple.can_attempt_breakthrough() {
                rows.push("Ready to seek a breakthrough. Open Orders to attempt it.".to_owned());
            }
            rows.push(format!(
                "Body {} / Mind {} / Spirit {}",
                disciple.attributes.body, disciple.attributes.mind, disciple.attributes.spirit
            ));
        } else {
            rows.push(format!(
                "Cultivation law: {}",
                disciple
                    .law_id
                    .as_ref()
                    .and_then(|id| data.laws.get(id))
                    .map(|law| law.name.as_str())
                    .unwrap_or("Unassigned")
            ));
            rows.push(format!(
                "Talent: {} / Bound artifacts: {} of 10",
                disciple.talent,
                disciple.equipment.len()
            ));
            if let Some(bloodline) = disciple
                .bloodline
                .bloodline_id
                .as_ref()
                .and_then(|id| data.bloodlines.get(id))
            {
                rows.push(format!(
                    "Bloodline: {} / {:.0}% awakened",
                    bloodline.name,
                    disciple.bloodline.awakening_progress * 100.0
                ));
            }
            rows.extend(
                disciple
                    .fate_traits
                    .iter()
                    .map(|fate| format!("Fate mark: {}", fate.name)),
            );
        }
        ux::text_pages(body, &rows, &mut self.detail_page);
        None
    }
    fn draw_orders(
        &mut self,
        rect: Rect,
        data: &GameData,
        disciple: &Disciple,
        index: usize,
    ) -> Option<UpdateResult> {
        let mut y = rect.y;
        let can_breakthrough =
            disciple.can_attempt_breakthrough() && disciple.breakthrough_bottleneck.is_none();
        let label = if can_breakthrough {
            "Seek Breakthrough"
        } else {
            "Breakthrough unavailable"
        };
        if ux::button(
            Rect::new(rect.x, y, rect.w, 46.0),
            label,
            true,
            can_breakthrough,
        ) {
            return Some(UpdateResult::new().with_action(Action::AttemptBreakthrough(index)));
        }
        y += 56.0;
        let half = (rect.w - 8.0) / 2.0;
        if ux::button(
            Rect::new(rect.x, y, half, 46.0),
            "Use supplies",
            false,
            true,
        ) {
            self.item_modal_open = true;
            self.modal_page = 0;
        }
        if ux::button(
            Rect::new(rect.x + half + 8.0, y, half, 46.0),
            "Artifacts",
            false,
            true,
        ) {
            self.equip_modal_open = true;
            self.modal_page = 0;
        }
        y += 56.0;
        let can_assign = disciple.rank != DiscipleRank::Outer && disciple.realm != "Mortal";
        if ux::button(
            Rect::new(rect.x, y, rect.w, 46.0),
            "Assign cultivation law",
            false,
            can_assign,
        ) {
            self.law_modal_open = true;
            self.modal_page = 0;
        }
        y += 56.0;
        if disciple.rank == DiscipleRank::Outer {
            if ux::button(
                Rect::new(rect.x, y, rect.w, 46.0),
                "Promote / 100 stones",
                false,
                disciple.realm != "Mortal",
            ) {
                return Some(UpdateResult::new().with_action(Action::PromoteDisciple(index)));
            }
            y += 56.0;
        }
        let reason = if disciple.realm == "Mortal" {
            "Promotion and laws require the first trace of Qi.".to_owned()
        } else if let Some(bottleneck) = &disciple.breakthrough_bottleneck {
            bottleneck.player_description(data)
        } else {
            "Use supplies and equipment to prepare for the next gate.".to_owned()
        };
        ux::clipped(
            Rect::new(rect.x, y, rect.w, (rect.y + rect.h - y).max(0.0)),
            || {
                draw_wrapped_text(
                    &reason,
                    rect.x,
                    y + 20.0,
                    rect.w,
                    FONT_SMALL_SIZE,
                    TEXT_SECONDARY,
                );
            },
        );
        None
    }
}
