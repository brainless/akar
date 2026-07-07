use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};
use crate::color::color_to_f32;
use crate::AkarTheme;
use crate::label::label;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BadgeVariant {
    Default,
    Primary,
    Success,
    Warning,
    Error,
    Info,
}

pub fn badge(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    text: &str,
    variant: BadgeVariant,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    let (bg, fg) = match variant {
        BadgeVariant::Default  => (theme.base_300,   theme.base_content),
        BadgeVariant::Primary  => (theme.primary,     theme.primary_content),
        BadgeVariant::Success  => (theme.success,     theme.success_content),
        BadgeVariant::Warning  => (theme.warning,     theme.warning_content),
        BadgeVariant::Error    => (theme.error,       theme.error_content),
        BadgeVariant::Info     => (theme.info,        theme.info_content),
    };

    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(bg),
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

    label(core, layout, node_id, text, fg, theme);
}
