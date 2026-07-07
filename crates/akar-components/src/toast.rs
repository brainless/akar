use akar_core::{AkarCore, QuadCall, TextCall, Z_OVERLAY};

use crate::color::color_to_f32;
use crate::AkarTheme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    Info,
    Success,
    Warning,
    Error,
}

pub struct ToastItem {
    pub variant: ToastVariant,
    pub message: String,
    pub dismiss_on_click: bool,
}

pub struct ToastResponse {
    pub dismissed: Option<usize>,
}

pub fn toasts(
    core: &mut AkarCore,
    viewport_rect: [f32; 4],
    items: &mut [ToastItem],
    theme: &AkarTheme,
) -> ToastResponse {
    if items.is_empty() {
        return ToastResponse { dismissed: None };
    }

    let [vx, vy, vw, vh] = viewport_rect;
    let viewport_right = vx + vw;
    let viewport_bottom = vy + vh;
    let padding_h = 16.0;
    let padding_v = 12.0;
    let gap = 8.0;
    let max_width = (vw * 0.35).clamp(0.0, 360.0);
    let text_max_w = (max_width - padding_h * 2.0).max(0.0);

    let metrics = glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2);

    let mut dismissed = None;
    let mut y = viewport_bottom - 16.0;

    for (i, item) in items.iter().enumerate() {
        let buffer_id = core.text_pipeline.set_text(
            Some(i as u64),
            &item.message,
            metrics,
            Some(text_max_w),
            None,
        );

        let text_size = core.text_pipeline.measure(buffer_id, None);
        let toast_h = text_size.y + padding_v * 2.0;
        let toast_w = max_width;

        y -= toast_h;
        let toast_rect = [viewport_right - toast_w - 16.0, y, toast_w, toast_h];

        let accent_color = match item.variant {
            ToastVariant::Info => theme.info,
            ToastVariant::Success => theme.success,
            ToastVariant::Warning => theme.warning,
            ToastVariant::Error => theme.error,
        };

        core.draw_list.push_quad(QuadCall {
            rect: toast_rect,
            fill: color_to_f32(accent_color),
            border_color: [0.0; 4],
            corner_radii: [theme.radius_field; 4],
            border_width: 0.0,
            z: Z_OVERLAY,
            shadow_blur: 8.0,
            shadow_spread: 0.0,
            shadow_color: [0.0, 0.0, 0.0, 0.3],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let text_x = toast_rect[0] + padding_h;
        let text_y = y + padding_v;
        core.draw_list.push_text(TextCall {
            buffer_id,
            x: text_x,
            y: text_y,
            clip: [text_x, text_y, text_max_w, text_size.y],
            color: color_to_f32(0xFFFFFFFF),
            z: Z_OVERLAY,
        });

        if dismissed.is_none() && item.dismiss_on_click && core.input.is_clicked(toast_rect) {
            dismissed = Some(i);
        }

        y -= gap;
    }

    ToastResponse { dismissed }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;

    fn make_core() -> AkarCore {
        AkarCore::mock()
    }

    #[test]
    fn empty_toasts_returns_no_dismissed() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        let mut items = Vec::new();
        let resp = toasts(
            &mut core,
            [0.0, 0.0, 800.0, 600.0],
            &mut items,
            &AKAR_THEME_DARK,
        );

        assert_eq!(core.draw_list.len(), 0);
        assert_eq!(resp.dismissed, None);
    }

    #[test]
    fn single_toast_renders_one_quad_and_text() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        let mut items = vec![ToastItem {
            variant: ToastVariant::Info,
            message: "Hello".into(),
            dismiss_on_click: false,
        }];

        toasts(
            &mut core,
            [0.0, 0.0, 800.0, 600.0],
            &mut items,
            &AKAR_THEME_DARK,
        );

        assert!(
            core.draw_list.len() >= 2,
            "expected >=2 calls, got {}",
            core.draw_list.len()
        );
    }

    #[test]
    fn multiple_toasts_stack_vertically() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        let mut items = vec![
            ToastItem {
                variant: ToastVariant::Info,
                message: "First toast".into(),
                dismiss_on_click: false,
            },
            ToastItem {
                variant: ToastVariant::Success,
                message: "Second toast".into(),
                dismiss_on_click: false,
            },
            ToastItem {
                variant: ToastVariant::Warning,
                message: "Third toast".into(),
                dismiss_on_click: false,
            },
        ];

        toasts(
            &mut core,
            [0.0, 0.0, 800.0, 600.0],
            &mut items,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads.len(), 3);

        for q in &quads {
            assert_eq!(q.rect[0] + q.rect[2], 784.0);
        }

        let mut y_vals: Vec<f32> = quads.iter().map(|q| q.rect[1]).collect();
        y_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        y_vals.dedup();
        assert_eq!(y_vals.len(), 3);
    }

    #[test]
    fn dismiss_on_click_returns_index() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        core.input.set_mouse_pos(600.0, 583.0);
        core.input.push_mouse_button(0, true);
        core.input.begin_frame();
        core.input.push_mouse_button(0, false);

        let mut items = vec![ToastItem {
            variant: ToastVariant::Info,
            message: "Click me".into(),
            dismiss_on_click: true,
        }];

        let resp = toasts(
            &mut core,
            [0.0, 0.0, 800.0, 600.0],
            &mut items,
            &AKAR_THEME_DARK,
        );

        assert_eq!(resp.dismissed, Some(0));
    }

    #[test]
    fn non_dismissable_toast_does_not_return_index() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        core.input.set_mouse_pos(600.0, 583.0);
        core.input.push_mouse_button(0, true);
        core.input.begin_frame();
        core.input.push_mouse_button(0, false);

        let mut items = vec![ToastItem {
            variant: ToastVariant::Info,
            message: "Click me".into(),
            dismiss_on_click: false,
        }];

        let resp = toasts(
            &mut core,
            [0.0, 0.0, 800.0, 600.0],
            &mut items,
            &AKAR_THEME_DARK,
        );

        assert_eq!(resp.dismissed, None);
    }

    #[test]
    fn all_variants_render_with_distinct_colors() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);

        let mut items = vec![
            ToastItem {
                variant: ToastVariant::Info,
                message: "Info".into(),
                dismiss_on_click: false,
            },
            ToastItem {
                variant: ToastVariant::Success,
                message: "Success".into(),
                dismiss_on_click: false,
            },
            ToastItem {
                variant: ToastVariant::Warning,
                message: "Warning".into(),
                dismiss_on_click: false,
            },
            ToastItem {
                variant: ToastVariant::Error,
                message: "Error".into(),
                dismiss_on_click: false,
            },
        ];

        toasts(
            &mut core,
            [0.0, 0.0, 800.0, 600.0],
            &mut items,
            &AKAR_THEME_DARK,
        );

        assert!(
            core.draw_list.len() >= 8,
            "expected >=8 calls, got {}",
            core.draw_list.len()
        );
    }
}
