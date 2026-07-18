use akar_core::AkarCore;
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
    _style: &DataItemStyle,
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
}
