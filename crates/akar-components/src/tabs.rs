use akar_core::AkarCore;
use akar_core::{QuadCall, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::AkarTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabVariant {
    Boxed,
    Lifted,
    Pills,
    Underline,
}

pub struct TabBarResponse {
    pub clicked: Option<usize>,
}

pub struct TabBarStyle {
    pub active_color: Option<u32>,
    pub inactive_color: Option<u32>,
    pub indicator_color: Option<u32>,
}

impl TabBarStyle {
    pub fn empty() -> Self {
        Self {
            active_color: None,
            inactive_color: None,
            indicator_color: None,
        }
    }
}

pub fn tab_bar(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    labels: &[&str],
    active_index: usize,
    variant: TabVariant,
    theme: &AkarTheme,
) -> TabBarResponse {
    let style = TabBarStyle::empty();
    tab_bar_styled(
        core,
        layout,
        node_id,
        labels,
        active_index,
        variant,
        &style,
        theme,
    )
}

pub fn tab_bar_styled(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    labels: &[&str],
    active_index: usize,
    variant: TabVariant,
    style: &TabBarStyle,
    theme: &AkarTheme,
) -> TabBarResponse {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return TabBarResponse { clicked: None };
    }

    if labels.is_empty() {
        return TabBarResponse { clicked: None };
    }

    let count = labels.len();
    let tab_width = rect[2] / count as f32;

    let active_color = style.active_color.unwrap_or(theme.primary);
    let inactive_color = style.inactive_color.unwrap_or(theme.base_200);
    let indicator_color = style.indicator_color.unwrap_or(theme.primary);

    let mut clicked = None;
    for i in 0..count {
        let tab_rect = [rect[0] + i as f32 * tab_width, rect[1], tab_width, rect[3]];
        if core.input.is_clicked(tab_rect) {
            clicked = Some(i);
        }
    }

    let active = clicked.unwrap_or(active_index).min(count - 1);

    for (i, label) in labels.iter().enumerate() {
        let tab_rect = [rect[0] + i as f32 * tab_width, rect[1], tab_width, rect[3]];

        let is_active = i == active;

        match variant {
            TabVariant::Boxed => {
                let (fill, border) = if is_active {
                    (active_color, active_color)
                } else {
                    (
                        inactive_color,
                        style
                            .inactive_color
                            .map(|_| inactive_color)
                            .unwrap_or(theme.base_300),
                    )
                };
                core.draw_list.push_quad(QuadCall {
                    rect: tab_rect,
                    fill: color_to_f32(fill),
                    border_color: color_to_f32(border),
                    corner_radii: [theme.radius_field; 4],
                    border_width: theme.border_width,
                    z: 0.0,
                    shadow_blur: 0.0,
                    shadow_spread: 0.0,
                    shadow_color: [0.0; 4],
                    shadow_offset: [0.0; 2],
                    _pad: [0.0; 2],
                });
            }
            TabVariant::Lifted => {
                let fill = if is_active {
                    theme.base_100
                } else {
                    inactive_color
                };
                core.draw_list.push_quad(QuadCall {
                    rect: tab_rect,
                    fill: color_to_f32(fill),
                    border_color: [0.0; 4],
                    corner_radii: [theme.radius_field, theme.radius_field, 0.0, 0.0],
                    border_width: 0.0,
                    z: 0.0,
                    shadow_blur: 0.0,
                    shadow_spread: 0.0,
                    shadow_color: [0.0; 4],
                    shadow_offset: [0.0; 2],
                    _pad: [0.0; 2],
                });
            }
            TabVariant::Pills => {
                let fill = if is_active {
                    active_color
                } else {
                    inactive_color
                };
                core.draw_list.push_quad(QuadCall {
                    rect: tab_rect,
                    fill: color_to_f32(fill),
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
            }
            TabVariant::Underline => {
                if is_active {
                    let underline_rect = [
                        tab_rect[0],
                        tab_rect[1] + tab_rect[3] - 3.0,
                        tab_rect[2],
                        3.0,
                    ];
                    core.draw_list.push_quad(QuadCall {
                        rect: underline_rect,
                        fill: color_to_f32(indicator_color),
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
                }
            }
        }

        let buffer_id = core.text_pipeline.set_text(
            Some(layout.widget_id(node_id) + i as u64),
            label,
            glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
            Some(tab_width),
            None,
            None,
        );

        let text_color = match (variant, is_active) {
            (TabVariant::Pills, true) => color_to_f32(theme.primary_content),
            (TabVariant::Boxed, true) => color_to_f32(theme.primary_content),
            (TabVariant::Underline, true) => color_to_f32(active_color),
            _ => color_to_f32(theme.base_content),
        };

        core.draw_list.push_text(TextCall {
            buffer_id,
            x: tab_rect[0] + theme.padding_x,
            y: tab_rect[1] + theme.padding_y,
            clip: tab_rect,
            color: text_color,
            z: 0.0,
        });
    }

    TabBarResponse { clicked }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::{length, Layout, Size, Style};

    fn node_400x40(layout: &mut Layout) -> NodeId {
        let n = layout.new_leaf(Style {
            size: Size {
                width: length(400.0),
                height: length(40.0),
            },
            ..Default::default()
        });
        layout.compute(n, (Some(400.0), Some(40.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        n
    }

    #[test]
    fn zero_labels_returns_no_click() {
        let mut layout = Layout::new();
        let node = node_400x40(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let result = tab_bar(
            &mut core,
            &layout,
            node,
            &[],
            0,
            TabVariant::Boxed,
            &AKAR_THEME_DARK,
        );

        assert_eq!(core.draw_list.len(), 0);
        assert_eq!(result.clicked, None);
    }

    #[test]
    fn renders_all_tabs_boxed() {
        let mut layout = Layout::new();
        let node = node_400x40(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        tab_bar(
            &mut core,
            &layout,
            node,
            &["Tab A", "Tab B", "Tab C"],
            1,
            TabVariant::Boxed,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads.len(), 3);

        let primary = color_to_f32(AKAR_THEME_DARK.primary);
        let base_200 = color_to_f32(AKAR_THEME_DARK.base_200);
        assert_eq!(quads[0].fill, base_200, "inactive tab 0");
        assert_eq!(quads[1].fill, primary, "active tab 1");
        assert_eq!(quads[2].fill, base_200, "inactive tab 2");
    }

    #[test]
    fn click_detection_on_first_tab() {
        let mut layout = Layout::new();
        let node = node_400x40(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let rect = layout.rect(node);
        let tab_x = rect[0];
        let tab_y = rect[1];

        core.input.set_mouse_pos(tab_x + 1.0, tab_y + 1.0);
        core.input.push_mouse_button(0, true);
        core.input.begin_frame();
        core.input.push_mouse_button(0, false);

        let result = tab_bar(
            &mut core,
            &layout,
            node,
            &["First", "Second", "Third"],
            0,
            TabVariant::Boxed,
            &AKAR_THEME_DARK,
        );

        assert_eq!(result.clicked, Some(0));
    }

    #[test]
    fn all_variants_no_panic() {
        let variants = [
            TabVariant::Boxed,
            TabVariant::Lifted,
            TabVariant::Pills,
            TabVariant::Underline,
        ];

        for variant in variants {
            let mut layout = Layout::new();
            let node = node_400x40(&mut layout);
            let mut core = AkarCore::mock();
            core.draw_list.begin_frame(1.0);

            tab_bar(
                &mut core,
                &layout,
                node,
                &["Alpha", "Beta"],
                0,
                variant,
                &AKAR_THEME_DARK,
            );

            assert!(
                !core.draw_list.is_empty(),
                "variant {:?} produced no draw calls",
                variant,
            );
        }
    }

    #[test]
    fn styled_uses_custom_fill() {
        let mut layout = Layout::new();
        let node = node_400x40(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let style = TabBarStyle {
            active_color: Some(0xFF0000FF),
            ..TabBarStyle::empty()
        };

        tab_bar_styled(
            &mut core,
            &layout,
            node,
            &["A", "B"],
            0,
            TabVariant::Boxed,
            &style,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert!(!quads.is_empty());
        assert_eq!(quads[0].fill, color_to_f32(0xFF0000FF));
    }

    #[test]
    fn styled_preserves_zero_area() {
        let mut layout = Layout::new();
        let node = node_400x40(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        let style = TabBarStyle {
            active_color: Some(0xFF0000FF),
            ..TabBarStyle::empty()
        };

        let result = tab_bar_styled(
            &mut core,
            &layout,
            node,
            &[],
            0,
            TabVariant::Boxed,
            &style,
            &AKAR_THEME_DARK,
        );

        assert_eq!(result.clicked, None);
    }
}
