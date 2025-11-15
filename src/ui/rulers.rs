// use crate::ui::bottom_panel::bottom_panel;
use crate::view_model::{BAPViewModel, RulerOrigin};
use eframe::egui;
use egui::{Align2, Color32, FontId, Painter, Rect, Response, Stroke, StrokeKind, Ui, pos2, vec2};

pub(crate) fn draw_rulers(
    model: &mut BAPViewModel,
    ui: &Ui,
    ctx: &egui::Context,
    painter: &Painter,
    painter_resp: &Response,
) {
    // This is the ruler display
    if model.show_rulers {
        let p1 = painter_resp.rect.min;
        let p2 = painter_resp.rect.max;
        let p3 = pos2(p2.x, p1.y + 16.);
        let p4 = pos2(p1.x, p1.y + 16.);
        let p5 = pos2(p1.x + 16., p2.y);
        let color = ui.visuals().text_edit_bg_color(); //.faint_bg_color.clone();
        let top_rect = Rect::from_min_max(p1, p3);
        let left_rect = Rect::from_min_max(p4, p5);
        painter.rect(top_rect, 0., color, Stroke::NONE, StrokeKind::Outside);
        painter.rect(left_rect, 0., color, Stroke::NONE, StrokeKind::Outside);

        // Then the pips
        let scale = model.scale_mm_to_screen(vec2(1., 0.)).x;
        let (ruler_major, ruler_minor, minor_count) = if scale > 20. {
            (1., 0.2, 4usize)
        } else if scale > 10. {
            (2., 0.5, 3usize)
        } else if scale > 4. {
            (5., 1., 4)
        } else if scale > 2. {
            (10., 2., 4)
        } else if scale > 1. {
            (20., 5., 3)
        } else if scale > 1. / 2.5 {
            (50., 10., 4)
        } else {
            (100., 20., 4)
        };

        let (mut major_x, mut major_y) = match model.ruler_origin {
            RulerOrigin::Source => (0., 0.),
            RulerOrigin::Origin => (model.origin.x, model.origin.y),
        };

        let rofs = match model.ruler_origin {
            RulerOrigin::Origin => model.origin - pos2(0., 0.),
            RulerOrigin::Source => vec2(0., 0.),
        };

        let right_limit = painter_resp.rect.right();
        let left_limit = painter_resp.rect.left();
        let _right_of_y_bar = painter_resp.rect.left();
        let top_limit = painter_resp.rect.top();
        let bottom_limit = painter_resp.rect.bottom();
        let bottom_of_x_bar = painter_resp.rect.top() + 16.;
        let color = ui.visuals().text_color();
        let mm_right = model.frame_coords_to_mm(pos2(right_limit, 0.)).x;
        let _mm_left = model.frame_coords_to_mm(pos2(left_limit, 0.)).x;
        let _mm_bottom = model.frame_coords_to_mm(pos2(0., bottom_limit)).y;
        let mm_top = model.frame_coords_to_mm(pos2(0., top_limit + 16.)).y;
        let minor_inc = model.scale_mm_to_screen(vec2(ruler_minor, 0.)).x;

        // X Axis ruler positive.
        while major_x < mm_right {
            let xpos = model.mm_to_frame_coords(pos2(major_x, 0.)).x;
            painter.line_segment(
                [pos2(xpos, top_limit), pos2(xpos, bottom_of_x_bar)],
                Stroke {
                    width: 1.,
                    color: color,
                },
            );
            for i in 1..=minor_count {
                painter.line_segment(
                    [
                        pos2(xpos + (i as f32 * minor_inc), bottom_of_x_bar - 4.),
                        pos2(xpos + (i as f32 * minor_inc), bottom_of_x_bar),
                    ],
                    Stroke {
                        width: 1.,
                        color: color,
                    },
                );
            }
            painter.text(
                pos2(xpos + 2., top_limit),
                Align2::LEFT_TOP,
                format!("{:3.1}", major_x - rofs.x),
                FontId::proportional(6.),
                color,
            );
            major_x += ruler_major;
        }

        // Y axis ruler positive
        while major_y < bottom_limit {
            let ypos = model.mm_to_frame_coords(pos2(0., major_y)).y;
            painter.line_segment(
                [pos2(left_limit, ypos), pos2(left_limit + 16., ypos)],
                Stroke {
                    width: 1.,
                    color: color,
                },
            );
            painter.text(
                pos2(left_limit, ypos + 1.),
                Align2::LEFT_TOP,
                format!("{:3.1}", major_y - rofs.y),
                FontId::proportional(6.),
                color,
            );
            for i in 1..=minor_count {
                painter.line_segment(
                    [
                        pos2(left_limit + 12.0, ypos + (i as f32 * minor_inc)),
                        pos2(left_limit + 16., ypos + (i as f32 * minor_inc)),
                    ],
                    Stroke {
                        width: 1.,
                        color: color,
                    },
                );
            }
            major_y += ruler_major;
        }

        let (mut major_x, major_y) = match model.ruler_origin {
            RulerOrigin::Source => (0., 0.),
            RulerOrigin::Origin => (model.origin.x, model.origin.y),
        };
        major_x = major_x - ruler_major;
        let mm_left = model.frame_coords_to_mm(pos2(left_limit, 0.)).x;
        while major_x > mm_left {
            let xpos = model.mm_to_frame_coords(pos2(major_x, 0.)).x;
            painter.line_segment(
                [pos2(xpos, top_limit), pos2(xpos, bottom_of_x_bar)],
                Stroke {
                    width: 1.,
                    color: color,
                },
            );
            for i in 1..=minor_count {
                painter.line_segment(
                    [
                        pos2(xpos + (i as f32 * minor_inc), bottom_of_x_bar - 4.),
                        pos2(xpos + (i as f32 * minor_inc), bottom_of_x_bar),
                    ],
                    Stroke {
                        width: 1.,
                        color: color,
                    },
                );
            }
            painter.text(
                pos2(xpos + 2., top_limit),
                Align2::LEFT_TOP,
                format!("{:3.1}", major_x - rofs.x),
                FontId::proportional(6.),
                color,
            );
            major_x -= ruler_major;
        }

        // Y axis ruler negative
        let mut major_y = major_y - ruler_major;
        while major_y > mm_top {
            let ypos = model.mm_to_frame_coords(pos2(0., major_y)).y;
            painter.line_segment(
                [pos2(left_limit, ypos), pos2(left_limit + 16., ypos)],
                Stroke {
                    width: 1.,
                    color: color,
                },
            );
            for i in 1..=minor_count {
                painter.line_segment(
                    [
                        pos2(left_limit + 12.0, ypos + (i as f32 * minor_inc)),
                        pos2(left_limit + 16., ypos + (i as f32 * minor_inc)),
                    ],
                    Stroke {
                        width: 1.,
                        color: color,
                    },
                );
            }
            painter.text(
                pos2(left_limit, ypos + 1.),
                Align2::LEFT_TOP,
                format!("{:3.1}", major_y - rofs.y),
                FontId::proportional(6.),
                color,
            );
            major_y -= ruler_major;
        }

        // Done the ruler, now just an overlay in red with current position.
        let _color = ui.visuals().strong_text_color();
        if let Some(lpos) = ctx.pointer_latest_pos() {
            painter.line_segment(
                [pos2(lpos.x, top_limit), pos2(lpos.x, bottom_of_x_bar)],
                Stroke {
                    width: 1.,
                    // color: color,
                    color: Color32::RED,
                },
            );
            painter.line_segment(
                [pos2(left_limit, lpos.y), pos2(left_limit + 16., lpos.y)],
                Stroke {
                    width: 1.,
                    // color: color,
                    color: Color32::RED,
                },
            );
        }

        (Some(top_rect), Some(left_rect))
    } else {
        (None, None)
    };
}
