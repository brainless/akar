use crate::color::color_to_f32;
use crate::AkarTheme;
use akar_core::{AkarCore, QuadCall, TextCall};
use akar_layout::{Layout, NodeId};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AlertVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

pub struct AlertResult {
    pub dismissed: bool,
}

fn dim_color(c: u32, factor: f32) -> u32 {
    let r = (((c >> 24) & 0xFF) as f32 * factor) as u32;
    let g = (((c >> 16) & 0xFF) as f32 * factor) as u32;
    let b = (((c >> 8) & 0xFF) as f32 * factor) as u32;
    let a = c & 0xFF;
    (r.min(255) << 24) | (g.min(255) << 16) | (b.min(255) << 8) | a
}

pub fn alert(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    text: &str,
    variant: AlertVariant,
    closable: bool,
    theme: &AkarTheme,
) -> AlertResult {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return AlertResult { dismissed: false };
    }

    let [x, y, w, h] = rect;

    let accent_color = match variant {
        AlertVariant::Info => theme.info,
        AlertVariant::Success => theme.success,
        AlertVariant::Warning => theme.warning,
        AlertVariant::Error => theme.error,
    };

    let border_color = dim_color(accent_color, 0.5);

    core.draw_list.push_quad(QuadCall {
        rect: [x, y, w, h],
        fill: color_to_f32(theme.base_200),
        border_color: color_to_f32(border_color),
        corner_radii: [theme.radius_field; 4],
        border_width: theme.border_width,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let strip_w = 4.0;
    core.draw_list.push_quad(QuadCall {
        rect: [x, y, strip_w, h],
        fill: color_to_f32(accent_color),
        border_color: [0.0; 4],
        corner_radii: [0.0; 4],
        border_width: 0.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let icon_size = 20.0;
    let icon_x = x + strip_w + theme.padding_x;
    let icon_y = y + (h - icon_size) / 2.0;
    core.draw_list.push_quad(QuadCall {
        rect: [icon_x, icon_y, icon_size, icon_size],
        fill: color_to_f32(accent_color),
        border_color: [0.0; 4],
        corner_radii: [theme.radius_field; 4],
        border_width: 0.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let close_btn_size = 24.0;
    let close_x = x + w - close_btn_size - theme.padding_x;
    let close_y = y + (h - close_btn_size) / 2.0;
    let mut dismissed = false;

    if closable {
        let close_rect = [close_x, close_y, close_btn_size, close_btn_size];
        if core.input.is_clicked(close_rect) {
            dismissed = true;
        }

        let key: u64 = node_id.into();
        let close_buffer_id = core.text_pipeline.set_text(
            Some(key.wrapping_add(1)),
            "×",
            glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
            Some(close_btn_size),
            None,
        );

        core.draw_list.push_text(TextCall {
            buffer_id: close_buffer_id,
            x: close_x + (close_btn_size - theme.font_size_base) / 2.0,
            y: y + (h - theme.font_size_base * 1.2) / 2.0,
            clip: close_rect,
            color: color_to_f32(theme.base_content),
            z: 0.0,
        });
    }

    let gap_after_icon = 8.0;
    let text_x = icon_x + icon_size + gap_after_icon;
    let text_y = y + (h - theme.font_size_base * 1.2) / 2.0;
    let right_boundary = if closable {
        close_x - theme.padding_x
    } else {
        x + w - theme.padding_x
    };
    let text_max_w = right_boundary - text_x;

    if text_max_w > 0.0 {
        let msg_buffer_id = core.text_pipeline.set_text(
            Some(node_id.into()),
            text,
            glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
            Some(text_max_w),
            None,
        );

        core.draw_list.push_text(TextCall {
            buffer_id: msg_buffer_id,
            x: text_x,
            y: text_y,
            clip: [text_x, y, text_max_w, h],
            color: color_to_f32(theme.base_content),
            z: 0.0,
        });
    }

    AlertResult { dismissed }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    fn sized_node(layout: &mut akar_layout::Layout) -> akar_layout::NodeId {
        let node = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(400.0),
                height: akar_layout::length(60.0),
            },
            ..Default::default()
        });
        layout.compute(node, (Some(400.0), Some(60.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        node
    }

    #[test]
    fn zero_area_returns_default() {
        let mut layout = akar_layout::Layout::new();
        let node_id = layout.new_leaf(Style::default());

        let mut core = AkarCore::mock();

        let result = alert(
            &mut core,
            &layout,
            node_id,
            "Test message",
            AlertVariant::Info,
            true,
            &AKAR_THEME_DARK,
        );

        assert!(!result.dismissed);
        assert_eq!(core.draw_list.len(), 0);
    }

    #[test]
    fn info_variant_uses_theme_info() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        alert(
            &mut core,
            &layout,
            node,
            "Test message",
            AlertVariant::Info,
            false,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        let info_color = color_to_f32(AKAR_THEME_DARK.info);
        assert!(quads.iter().any(|q| q.fill == info_color));
    }

    #[test]
    fn closable_alert_has_close_button() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);

        let mut core1 = AkarCore::mock();
        alert(
            &mut core1,
            &layout,
            node,
            "Test",
            AlertVariant::Info,
            false,
            &AKAR_THEME_DARK,
        );
        let count_without = core1.draw_list.len();

        let mut core2 = AkarCore::mock();
        alert(
            &mut core2,
            &layout,
            node,
            "Test",
            AlertVariant::Info,
            true,
            &AKAR_THEME_DARK,
        );
        let count_with = core2.draw_list.len();

        assert!(count_with > count_without);
    }

    #[test]
    fn close_click_dismisses() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();

        let rect = layout.rect(node);
        let [x, y, w, h] = rect;
        let close_btn_size = 24.0;
        let close_x = x + w - close_btn_size - AKAR_THEME_DARK.padding_x;
        let close_y = y + (h - close_btn_size) / 2.0;

        core.input
            .set_mouse_pos(close_x + close_btn_size / 2.0, close_y + close_btn_size / 2.0);
        core.input.push_mouse_button(0, true);
        core.input.push_mouse_button(0, false);

        let result = alert(
            &mut core,
            &layout,
            node,
            "Test",
            AlertVariant::Info,
            true,
            &AKAR_THEME_DARK,
        );

        assert!(result.dismissed);
    }
}
