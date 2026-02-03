use egui::{CentralPanel, Context};

/// UI content for the control panel.
pub struct ControlPanel;

impl ControlPanel {
    pub fn new() -> Self {
        Self
    }

    #[expect(clippy::unused_self, reason = "will hold UI state")]
    pub fn show(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, World!");
        });
    }
}
