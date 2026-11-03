//! Separate display preferences and confirmation for fullscreen changes.

use macroquad::prelude::*;
use macroquad_toolkit::settings::{DisplayPreview, GameSettings};

use crate::ui::components::draw_panel;
use crate::ui::theme::{FONT_SMALL_SIZE, PRIMARY, TEXT_PRIMARY, TEXT_SECONDARY};
use macroquad_toolkit::ui::draw_ui_text;

const GAME_NAME: &str = "cultivation";

struct ActivePreview {
    candidate: GameSettings,
    preview: DisplayPreview,
}

pub(super) struct DisplaySettings {
    committed: GameSettings,
    active_preview: Option<ActivePreview>,
}

pub(super) enum DisplayPreviewAction {
    Keep,
    Revert,
}

impl DisplaySettings {
    pub(super) fn load(capture_mode: bool) -> Self {
        let mut committed = GameSettings::load(GAME_NAME);
        if capture_mode {
            committed.fullscreen = false;
        }
        #[cfg(target_arch = "wasm32")]
        {
            // Browsers require a user gesture before entering fullscreen.
            committed.fullscreen = false;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if committed.fullscreen && !capture_mode {
            set_fullscreen(true);
        }

        Self {
            committed,
            active_preview: None,
        }
    }

    pub(super) fn fullscreen(&self) -> bool {
        self.active_preview
            .as_ref()
            .map(|active| active.candidate.fullscreen)
            .unwrap_or(self.committed.fullscreen)
    }

    pub(super) fn begin_preview(&mut self) {
        if self.active_preview.is_some() {
            return;
        }
        let mut candidate = self.committed.clone();
        candidate.fullscreen = !candidate.fullscreen;
        let preview = DisplayPreview::begin(&self.committed, &candidate);
        self.active_preview = Some(ActivePreview { candidate, preview });
    }

    pub(super) fn is_previewing(&self) -> bool {
        self.active_preview.is_some()
    }

    pub(super) fn preview_remaining(&self) -> Option<f32> {
        self.active_preview
            .as_ref()
            .map(|active| active.preview.remaining())
    }

    /// Returns true after the toolkit restores the original display on timeout.
    pub(super) fn update(&mut self, real_dt: f32) -> bool {
        let expired = self
            .active_preview
            .as_mut()
            .map(|active| active.preview.update(real_dt))
            .unwrap_or(false);
        if expired {
            self.active_preview = None;
        }
        expired
    }

    pub(super) fn keep_preview(&mut self) -> Result<(), String> {
        let Some(mut active) = self.active_preview.take() else {
            return Ok(());
        };
        if let Err(error) = active.candidate.save(GAME_NAME) {
            active.preview.revert();
            return Err(error);
        }
        active.preview.confirm();
        self.committed = active.candidate;
        Ok(())
    }

    pub(super) fn revert_preview(&mut self) {
        if let Some(mut active) = self.active_preview.take() {
            active.preview.revert();
        }
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            committed: GameSettings::default(),
            active_preview: None,
        }
    }
}

pub(super) fn draw_preview(remaining: f32) -> Option<DisplayPreviewAction> {
    let sw = screen_width();
    let sh = screen_height();
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.78));

    let width = sw.clamp(360.0, 560.0);
    let panel = Rect::new((sw - width) * 0.5, (sh - 244.0) * 0.5, width, 244.0);
    draw_panel(panel, Some("Confirm Display"));
    draw_ui_text(
        "Keep this fullscreen setting?",
        panel.x + 24.0,
        panel.y + 94.0,
        FONT_SMALL_SIZE,
        TEXT_PRIMARY,
    );
    draw_ui_text(
        &format!("Reverting in {:.0} seconds", remaining.ceil()),
        panel.x + 24.0,
        panel.y + 126.0,
        FONT_SMALL_SIZE,
        TEXT_SECONDARY,
    );

    let button_y = panel.y + 166.0;
    let button_w = ((panel.w - 72.0) * 0.5).min(210.0);
    if crate::ui::components::draw_button(
        Rect::new(panel.x + 24.0, button_y, button_w, 44.0),
        "Keep Display",
        true,
    ) {
        return Some(DisplayPreviewAction::Keep);
    }
    if crate::ui::components::draw_button(
        Rect::new(
            panel.x + panel.w - button_w - 24.0,
            button_y,
            button_w,
            44.0,
        ),
        "Revert",
        false,
    ) {
        return Some(DisplayPreviewAction::Revert);
    }
    None
}

pub(super) fn draw_notice(message: &str) {
    let width = screen_width().clamp(320.0, 620.0);
    let rect = Rect::new(
        (screen_width() - width) * 0.5,
        screen_height() - 76.0,
        width,
        54.0,
    );
    draw_panel(rect, None);
    draw_ui_text(
        message,
        rect.x + 18.0,
        rect.y + 34.0,
        FONT_SMALL_SIZE,
        PRIMARY,
    );
}
