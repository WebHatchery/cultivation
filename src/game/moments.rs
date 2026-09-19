use super::Game;
use crate::ui::workspace as ux;
use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub(in crate::game) enum MomentKind {
    Breakthrough,
    Discovery,
    Recruitment,
    SpiritBeast,
    Tragedy,
    Founding,
}

pub(in crate::game) struct MomentOverlay {
    kind: MomentKind,
    title: String,
    subtitle: String,
    detail: String,
    age_frames: u32,
    page: usize,
}
impl Game {
    pub(in crate::game) fn show_moment(
        &mut self,
        kind: MomentKind,
        title: impl Into<String>,
        subtitle: impl Into<String>,
        detail: impl Into<String>,
    ) {
        self.active_moment = Some(MomentOverlay {
            kind,
            title: title.into(),
            subtitle: subtitle.into(),
            detail: detail.into(),
            age_frames: 0,
            page: 0,
        });
    }
    pub(in crate::game) fn update_moment_timer(&mut self) {
        if let Some(moment) = &mut self.active_moment {
            moment.age_frames = moment.age_frames.saturating_add(1);
        }
    }
    pub(in crate::game) fn draw_active_moment(&mut self) {
        let Some(moment) = &mut self.active_moment else {
            return;
        };
        ux::header_to(
            "A moment in the annals",
            "The sect is paused. Continue when you are ready.",
            "",
        );
        let area = ux::content();
        ux::text_pages(
            Rect::new(area.x, area.y, area.w, area.h - 72.0),
            &[
                moment_stamp_label(moment.kind).to_owned(),
                moment.title.clone(),
                moment.subtitle.clone(),
                moment.detail.clone(),
            ],
            &mut moment.page,
        );
        // The release that caused a milestone must not dismiss it in the same frame.
        if ux::button(
            Rect::new(area.x + area.w - 200.0, area.y + area.h - 52.0, 200.0, 48.0),
            "Continue",
            true,
            moment.age_frames > 1,
        ) {
            self.active_moment = None;
        }
    }
}

fn moment_stamp_label(kind: MomentKind) -> &'static str {
    match kind {
        MomentKind::Breakthrough => "ASCEND",
        MomentKind::Discovery => "FOUND",
        MomentKind::Recruitment => "ADMIT",
        MomentKind::SpiritBeast => "BIND",
        MomentKind::Tragedy => "FATE",
        MomentKind::Founding => "SECT",
    }
}
