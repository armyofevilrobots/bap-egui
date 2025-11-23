
use crate::BAPViewModel;
use crate::core::project::{Orientation, PaperSize};
use eframe::egui;
use egui::{Align2, Color32, ComboBox, FontId, Id, Layout, Rect, Stroke, pos2, vec2};

pub(crate) fn paper_chooser_window(
    model: &mut BAPViewModel,
    ctx: &egui::Context,
    // ui: &mut egui::Ui,
) /*-> ModalResponse<()>*/
{
    egui::Modal::new(Id::new("Paper Chooser")).show(ctx, |ui| {
        ui.set_width(400.);
        ui.heading("Paper Selection");
        let (painter_resp, painter) = ui.allocate_painter(vec2(400., 420.), egui::Sense::all());
        let cur = ui.cursor().min;
        let prect = painter_resp.rect;
        let (px, py) = model.paper_size().dimensions();
        let (px, py) = match model.paper_orientation() {
            Orientation::Landscape => (py, px),
            Orientation::Portrait => (px, py),
        };
        let ratio = py / px;
        let (pwidth, pheight) = if ratio > 1. {
            (300. / ratio, 300.)
        } else {
            (300., 300. * ratio)
        };

        // println!("Cursor: {:?}", cur);
        painter.rect(
            Rect::from_center_size(
                pos2(cur.x + prect.width() / 2., cur.y - prect.height() / 2.),
                vec2(pwidth as f32, pheight as f32),
            ),
            0.,
            // Color32::from_white_alpha(128),
            model.paper_color(),
            Stroke::new(1., Color32::from_black_alpha(128)),
            egui::StrokeKind::Inside,
        );

        let pcol = model.paper_color().to_tuple();
        let tcol = (
            ((pcol.0 as u32 + 85) % 255) as u8,
            ((pcol.0 as u32 + 85) % 255) as u8,
            ((pcol.0 as u32 + 85) % 255) as u8,
        );
        let dimensions_text_color = Color32::from_rgb(tcol.0, tcol.1, tcol.2);

        painter.text(
            pos2(cur.x + prect.width() / 2., cur.y - prect.height() / 2.),
            Align2::CENTER_CENTER,
            format!("{}\n{}", model.paper_size(), model.paper_orientation()),
            FontId::default(),
            dimensions_text_color.clone(),
        );

        let (paper_width_mm, paper_height_mm) = match model.paper_orientation() {
            Orientation::Portrait => model.paper_size().dims(),
            Orientation::Landscape => {
                let (w, h) = model.paper_size().dims();
                (h, w)
            }
        };

        painter.text(
            pos2(
                cur.x + prect.width() / 2. + pwidth as f32 / 2. - 5.,
                cur.y - prect.height() / 2.,
            ),
            Align2::RIGHT_CENTER,
            format!("{}mm", paper_height_mm), //model.paper_size.dims().0),
            FontId::proportional(8.),
            dimensions_text_color.clone(),
        );

        painter.text(
            pos2(
                cur.x + prect.width() / 2.,
                cur.y - prect.height() / 2. + pheight as f32 / 2. - 5.,
            ),
            Align2::CENTER_BOTTOM,
            format!("{}mm", paper_width_mm), //model.paper_size.dims().1),
            FontId::proportional(8.),
            dimensions_text_color.clone(),
        );
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            // if ui.button("Cancel").clicked() {
            //     // model.paper_modal_open = false
            //     model.command_context = crate::view_model::CommandContext::None;
            // }
            if ui.button("Ok").clicked() {
                // model.paper_modal_open = false
                model.command_context = crate::view_model::CommandContext::None;
            }

            let mut orientation = model.paper_orientation();
            paper_chooser_combobox(model, ui);
            ui.radio_value(&mut orientation, Orientation::Landscape, "Landscape");
            ui.radio_value(&mut orientation, Orientation::Portrait, "Portrait");
            if orientation != model.paper_orientation() {
                model.set_paper_orientation(&orientation, true);
            }
            let mut color_tmp = model.paper_color();
            ui.color_edit_button_srgba(&mut color_tmp);
            if color_tmp != model.paper_color() {
                model.set_paper_color(&mut color_tmp, true);
            }
        });
    });
}

pub(crate) fn paper_chooser_combobox(model: &mut BAPViewModel, ui: &mut egui::Ui) {
    let mut psize = model.paper_size();
    let mut changed = false;
    ComboBox::from_label("")
        .selected_text(format!("{}", model.paper_size()))
        .show_ui(ui, |ui| {
            for ps in PaperSize::all().iter() {
                if ui
                    .selectable_value(&mut psize, ps.clone(), format!("{}", ps))
                    .clicked()
                {
                    changed = true;
                };
            }
        });
    if changed {
        model.set_paper_size(&psize, true);
    }
}
