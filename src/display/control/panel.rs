use egui::{CentralPanel, Context};
use winit::event_loop::EventLoopProxy;

use crate::AppEvent;

/// UI content for the control panel.
pub struct ControlPanel {
    proxy: EventLoopProxy<AppEvent>,
}

impl ControlPanel {
    pub fn new(proxy: EventLoopProxy<AppEvent>) -> Self {
        Self { proxy }
    }

    pub fn show(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Qualia Control");

            if ui.button("Ping Trainer").clicked() {
                self.proxy
                    .send_event(AppEvent::FeedbackRequested)
                    .expect("event loop closed while handling UI event");
            }
        });
    }
}
