use crate::engine::search_stats::{ReportSection, ReportSettings};
use eframe::egui::{self, collapsing_header::CollapsingState};

pub(super) fn show(ui: &mut egui::Ui, settings: &mut ReportSettings) {
    ui.horizontal(|ui| {
        for (label, selection) in [
            ("Print stats", &mut settings.printed),
            ("Highlight stats", &mut settings.highlighted),
        ] {
            let button = ui.button(label);
            egui::Popup::menu(&button)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    egui::ScrollArea::vertical()
                        .max_height(320.0)
                        .show(ui, |ui| {
                            show_node(ui, ReportSection::All, selection);
                        });
                });
        }
    });
}

fn show_node(ui: &mut egui::Ui, node: ReportSection, selection: &mut u64) {
    let (label, children) = node.tree();
    let mask = node.mask();
    let mut checkbox = |ui: &mut egui::Ui| {
        let selected = *selection & mask;
        let mut checked = selected == mask;
        let partial = selected != 0 && !checked;
        if ui
            .add(egui::Checkbox::new(&mut checked, label).indeterminate(partial))
            .changed()
        {
            if checked {
                *selection |= mask;
            } else {
                *selection &= !mask;
            }
        }
    };
    if children.is_empty() {
        checkbox(ui);
    } else {
        CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id(node),
            node == ReportSection::All,
        )
        .show_header(ui, checkbox)
        .body(|ui| {
            for &child in children {
                show_node(ui, child, selection);
            }
        });
    }
}
