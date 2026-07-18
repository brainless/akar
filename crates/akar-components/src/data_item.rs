use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};

use crate::AkarTheme;

#[derive(Clone, Copy)]
pub struct DataItemStyle {
    pub surface: [f32; 4],
    pub padding_x: f32,
    pub padding_y: f32,
    pub spacing: f32,
    pub color_normal: [f32; 4],
    pub color_hover: [f32; 4],
    pub color_pressed: [f32; 4],
    pub color_selected: [f32; 4],
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
}

impl DataItemStyle {
    pub fn from_theme(theme: &AkarTheme) -> Self {
        use crate::color::{color_to_f32, scale_color};

        Self {
            surface: color_to_f32(theme.base_200),
            padding_x: theme.padding_x,
            padding_y: theme.padding_y,
            spacing: 8.0,
            color_normal: color_to_f32(theme.base_200),
            color_hover: color_to_f32(scale_color(theme.base_200, 1.15)),
            color_pressed: color_to_f32(scale_color(theme.base_200, 0.9)),
            color_selected: color_to_f32(theme.primary),
            corner_radius: theme.radius_field,
            border_width: theme.border_width,
            border_color: color_to_f32(theme.base_300),
        }
    }

    fn fill_for_state(&self, hovered: bool, pressed: bool) -> [f32; 4] {
        if pressed {
            self.color_pressed
        } else if hovered {
            self.color_hover
        } else {
            self.color_normal
        }
    }
}

impl Default for DataItemStyle {
    fn default() -> Self {
        Self::from_theme(&crate::AKAR_THEME_DARK)
    }
}

pub struct DataItemResponse {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
}

pub fn data_item(
    core: &mut AkarCore,
    layout: &Layout,
    node: NodeId,
    _key: u64,
    style: &DataItemStyle,
) -> DataItemResponse {
    let rect = layout.rect(node);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return DataItemResponse {
            hovered: false,
            pressed: false,
            clicked: false,
        };
    }

    let hovered = core.input.is_hovering(rect);
    let pressed = core.input.is_pressed(rect);
    let clicked = core.input.is_clicked(rect);

    let fill = style.fill_for_state(hovered, pressed);
    if fill[3] > 0.0 || (style.border_width > 0.0 && style.border_color[3] > 0.0) {
        core.draw_list.push_quad(QuadCall {
            rect,
            fill,
            border_color: style.border_color,
            corner_radii: [style.corner_radius; 4],
            border_width: style.border_width,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });
    }

    DataItemResponse {
        hovered,
        pressed,
        clicked,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::{length, Size, Style};

    fn sized_leaf(w: f32, h: f32) -> (Layout, NodeId) {
        let mut layout = Layout::new();
        let node = layout.new_leaf(Style {
            size: Size {
                width: length(w),
                height: length(h),
            },
            ..Default::default()
        });
        let root = layout.new_with_children(Style::default(), &[node]);
        layout.compute(root, (Some(400.0), Some(300.0)), |_, _, _, _, _| Size::ZERO);
        (layout, node)
    }

    #[test]
    fn zero_area_returns_all_false() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        let result = data_item(&mut core, &layout, node_id, 0, &DataItemStyle::default());

        assert!(!result.hovered);
        assert!(!result.pressed);
        assert!(!result.clicked);
    }

    #[test]
    fn hover_detected_inside_rect() {
        let (layout, node_id) = sized_leaf(100.0, 50.0);
        let mut core = AkarCore::mock();
        core.input.set_mouse_pos(50.0, 25.0);

        let result = data_item(
            &mut core,
            &layout,
            node_id,
            42,
            &DataItemStyle::from_theme(&AKAR_THEME_DARK),
        );

        assert!(result.hovered);
        assert!(!result.pressed);
        assert!(!result.clicked);
    }

    #[test]
    fn press_detected_inside_rect() {
        let (layout, node_id) = sized_leaf(100.0, 50.0);
        let mut core = AkarCore::mock();
        core.input.set_mouse_pos(50.0, 25.0);
        core.input.push_mouse_button(0, true);

        let result = data_item(&mut core, &layout, node_id, 7, &DataItemStyle::default());

        assert!(result.hovered);
        assert!(result.pressed);
    }

    #[test]
    fn click_detected_after_press_release() {
        let (layout, node_id) = sized_leaf(100.0, 50.0);
        let mut core = AkarCore::mock();
        core.input.set_mouse_pos(50.0, 25.0);
        core.input.push_mouse_button(0, true);
        core.input.begin_frame();
        core.input.set_mouse_pos(50.0, 25.0);
        core.input.push_mouse_button(0, false);

        let result = data_item(&mut core, &layout, node_id, 99, &DataItemStyle::default());

        assert!(result.clicked);
    }

    #[test]
    fn no_hover_outside_rect() {
        let (layout, node_id) = sized_leaf(100.0, 50.0);
        let mut core = AkarCore::mock();
        core.input.set_mouse_pos(200.0, 200.0);

        let result = data_item(&mut core, &layout, node_id, 1, &DataItemStyle::default());

        assert!(!result.hovered);
        assert!(!result.pressed);
        assert!(!result.clicked);
    }

    #[test]
    fn nonzero_rect_submits_one_quad() {
        let (layout, node_id) = sized_leaf(100.0, 50.0);
        let mut core = AkarCore::mock();

        data_item(&mut core, &layout, node_id, 0, &DataItemStyle::default());

        assert_eq!(core.draw_list.sorted_quads().len(), 1);
    }

    #[test]
    fn zero_area_submits_no_quads() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        data_item(&mut core, &layout, node_id, 0, &DataItemStyle::default());

        assert!(core.draw_list.sorted_quads().is_empty());
    }

    #[test]
    fn hover_changes_fill_color() {
        let (layout, node_id) = sized_leaf(100.0, 50.0);
        let style = DataItemStyle::default();

        let mut core_normal = AkarCore::mock();
        core_normal.input.set_mouse_pos(200.0, 200.0);
        data_item(&mut core_normal, &layout, node_id, 0, &style);
        let normal_fill = core_normal.draw_list.sorted_quads()[0].fill;

        let mut core_hover = AkarCore::mock();
        core_hover.input.set_mouse_pos(50.0, 25.0);
        data_item(&mut core_hover, &layout, node_id, 0, &style);
        let hover_fill = core_hover.draw_list.sorted_quads()[0].fill;

        assert_ne!(normal_fill, hover_fill);
    }

    #[test]
    fn pressed_changes_fill_color_vs_hover() {
        let (layout, node_id) = sized_leaf(100.0, 50.0);
        let style = DataItemStyle::default();

        let mut core_hover = AkarCore::mock();
        core_hover.input.set_mouse_pos(50.0, 25.0);
        data_item(&mut core_hover, &layout, node_id, 0, &style);
        let hover_fill = core_hover.draw_list.sorted_quads()[0].fill;

        let mut core_pressed = AkarCore::mock();
        core_pressed.input.set_mouse_pos(50.0, 25.0);
        core_pressed.input.push_mouse_button(0, true);
        data_item(&mut core_pressed, &layout, node_id, 0, &style);
        let pressed_fill = core_pressed.draw_list.sorted_quads()[0].fill;

        assert_ne!(hover_fill, pressed_fill);
    }

    #[test]
    fn transparent_style_submits_no_quad() {
        let (layout, node_id) = sized_leaf(100.0, 50.0);
        let mut core = AkarCore::mock();
        let style = DataItemStyle {
            surface: [0.0; 4],
            padding_x: 0.0,
            padding_y: 0.0,
            spacing: 0.0,
            color_normal: [0.0; 4],
            color_hover: [0.0; 4],
            color_pressed: [0.0; 4],
            color_selected: [0.0; 4],
            corner_radius: 0.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        };

        data_item(&mut core, &layout, node_id, 0, &style);

        assert!(core.draw_list.sorted_quads().is_empty());
    }

    #[test]
    fn border_only_style_submits_quad() {
        let (layout, node_id) = sized_leaf(100.0, 50.0);
        let mut core = AkarCore::mock();
        let style = DataItemStyle {
            surface: [0.0; 4],
            padding_x: 0.0,
            padding_y: 0.0,
            spacing: 0.0,
            color_normal: [0.0; 4],
            color_hover: [0.0; 4],
            color_pressed: [0.0; 4],
            color_selected: [0.0; 4],
            corner_radius: 0.0,
            border_width: 1.0,
            border_color: [1.0, 1.0, 1.0, 1.0],
        };

        data_item(&mut core, &layout, node_id, 0, &style);

        assert_eq!(core.draw_list.sorted_quads().len(), 1);
    }
}
