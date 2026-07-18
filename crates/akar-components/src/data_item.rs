use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId, WorldRect};

use crate::canvas::{CanvasAlign, CanvasInput, CanvasOverflow, CanvasPainter, CanvasTextStyle};
use crate::color::f32_to_color;
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

pub struct CanvasDataItemDescriptor<'a> {
    pub title: Option<&'a str>,
    pub supporting_text: Option<&'a str>,
    pub metadata: Option<&'a str>,
    pub style: &'a DataItemStyle,
}

pub struct CanvasDataItemResponse {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
}

pub fn canvas_data_item(
    painter: &mut CanvasPainter,
    input: &CanvasInput,
    world_rect: WorldRect,
    descriptor: &CanvasDataItemDescriptor,
) -> CanvasDataItemResponse {
    let hovered = input.is_hovering(world_rect);
    let pressed = input.is_pressed(world_rect);
    let clicked = input.is_clicked(world_rect);

    let style = descriptor.style;
    let fill = style.fill_for_state(hovered, pressed);
    let fill_u32 = f32_to_color(fill);

    painter.push_quad(
        world_rect,
        fill_u32,
        f32_to_color(style.border_color),
        style.border_width,
        [style.corner_radius; 4],
        0.0,
    );

    let content_x = world_rect.min.x + style.padding_x;
    let content_w = (world_rect.max.x - world_rect.min.x - style.padding_x * 2.0).max(0.0);
    let mut cursor_y = world_rect.min.y + style.padding_y;
    let title_font = 14.0;
    let supporting_font = 12.0;
    let metadata_font = 10.0;
    let line_height_factor = 1.3;
    let field_spacing = style.spacing;

    if let Some(title) = descriptor.title {
        let h = title_font * line_height_factor;
        let text_rect = WorldRect::from_xywh(content_x, cursor_y, content_w, h);
        painter.push_text(
            text_rect,
            title,
            &CanvasTextStyle {
                font_size: title_font,
                color: 0xFFFFFFFF,
                background: None,
                padding: [0.0; 4],
                align_x: CanvasAlign::Left,
                overflow: CanvasOverflow::Truncate,
            },
        );
        cursor_y += h + field_spacing;
    }

    if let Some(text) = descriptor.supporting_text {
        let h = supporting_font * line_height_factor;
        let text_rect = WorldRect::from_xywh(content_x, cursor_y, content_w, h);
        painter.push_text(
            text_rect,
            text,
            &CanvasTextStyle {
                font_size: supporting_font,
                color: 0xBBBBBBFF,
                background: None,
                padding: [0.0; 4],
                align_x: CanvasAlign::Left,
                overflow: CanvasOverflow::Truncate,
            },
        );
        cursor_y += h + field_spacing;
    }

    if let Some(text) = descriptor.metadata {
        let h = metadata_font * line_height_factor;
        let text_rect = WorldRect::from_xywh(content_x, cursor_y, content_w, h);
        painter.push_text(
            text_rect,
            text,
            &CanvasTextStyle {
                font_size: metadata_font,
                color: 0x999999FF,
                background: None,
                padding: [0.0; 4],
                align_x: CanvasAlign::Left,
                overflow: CanvasOverflow::Truncate,
            },
        );
    }

    CanvasDataItemResponse {
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

    fn make_canvas_painter() -> CanvasPainter {
        let w2s =
            akar_layout::make_world_to_screen(glam::Vec2::ZERO, 1.0, [0.0, 0.0, 800.0, 600.0]);
        CanvasPainter {
            quad_buffer: Vec::new(),
            text_buffer: Vec::new(),
            world_to_screen: w2s,
            canvas_rect: [0.0, 0.0, 800.0, 600.0],
            text_buffer_scope: 1,
        }
    }

    fn make_canvas_input(screen_x: f32, screen_y: f32) -> CanvasInput {
        let s2w =
            akar_layout::make_screen_to_world(glam::Vec2::ZERO, 1.0, [0.0, 0.0, 800.0, 600.0]);
        let mut input = akar_core::InputState::new();
        input.set_mouse_pos(screen_x, screen_y);
        CanvasInput::new(&input, &s2w)
    }

    fn make_canvas_input_with_click(screen_x: f32, screen_y: f32) -> CanvasInput {
        let s2w =
            akar_layout::make_screen_to_world(glam::Vec2::ZERO, 1.0, [0.0, 0.0, 800.0, 600.0]);
        let mut input = akar_core::InputState::new();
        input.set_mouse_pos(screen_x, screen_y);
        input.push_mouse_button(0, true);
        input.begin_frame();
        input.set_mouse_pos(screen_x, screen_y);
        input.push_mouse_button(0, false);
        CanvasInput::new(&input, &s2w)
    }

    #[test]
    fn canvas_data_item_hover_inside() {
        let mut painter = make_canvas_painter();
        let input = make_canvas_input(400.0, 300.0);
        let rect = WorldRect::from_xywh(-50.0, -50.0, 100.0, 100.0);
        let style = DataItemStyle::default();
        let desc = CanvasDataItemDescriptor {
            title: Some("Test"),
            supporting_text: None,
            metadata: None,
            style: &style,
        };

        let result = canvas_data_item(&mut painter, &input, rect, &desc);

        assert!(result.hovered);
        assert!(!result.pressed);
        assert!(!result.clicked);
    }

    #[test]
    fn canvas_data_item_no_hover_outside() {
        let mut painter = make_canvas_painter();
        let input = make_canvas_input(100.0, 100.0);
        let rect = WorldRect::from_xywh(-50.0, -50.0, 20.0, 20.0);
        let style = DataItemStyle::default();
        let desc = CanvasDataItemDescriptor {
            title: Some("Test"),
            supporting_text: None,
            metadata: None,
            style: &style,
        };

        let result = canvas_data_item(&mut painter, &input, rect, &desc);

        assert!(!result.hovered);
        assert!(!result.pressed);
        assert!(!result.clicked);
    }

    #[test]
    fn canvas_data_item_click() {
        let mut painter = make_canvas_painter();
        let input = make_canvas_input_with_click(400.0, 300.0);
        let rect = WorldRect::from_xywh(-50.0, -50.0, 100.0, 100.0);
        let style = DataItemStyle::default();
        let desc = CanvasDataItemDescriptor {
            title: Some("Test"),
            supporting_text: None,
            metadata: None,
            style: &style,
        };

        let result = canvas_data_item(&mut painter, &input, rect, &desc);

        assert!(result.hovered);
        assert!(result.clicked);
    }

    #[test]
    fn canvas_data_item_submits_quad_and_text() {
        let mut painter = make_canvas_painter();
        let input = make_canvas_input(400.0, 300.0);
        let rect = WorldRect::from_xywh(-50.0, -50.0, 100.0, 100.0);
        let style = DataItemStyle::default();
        let desc = CanvasDataItemDescriptor {
            title: Some("Title"),
            supporting_text: Some("Supporting"),
            metadata: Some("Meta"),
            style: &style,
        };

        canvas_data_item(&mut painter, &input, rect, &desc);

        assert_eq!(
            painter.quad_buffer.len(),
            1,
            "should submit one background quad"
        );
        assert_eq!(
            painter.text_buffer.len(),
            3,
            "should submit three text entries"
        );
    }

    #[test]
    fn canvas_data_item_partial_text_fields() {
        let mut painter = make_canvas_painter();
        let input = make_canvas_input(400.0, 300.0);
        let rect = WorldRect::from_xywh(-50.0, -50.0, 100.0, 100.0);
        let style = DataItemStyle::default();
        let desc = CanvasDataItemDescriptor {
            title: Some("Only title"),
            supporting_text: None,
            metadata: None,
            style: &style,
        };

        canvas_data_item(&mut painter, &input, rect, &desc);

        assert_eq!(painter.quad_buffer.len(), 1);
        assert_eq!(painter.text_buffer.len(), 1);
    }

    #[test]
    fn canvas_data_item_far_outside_still_works() {
        let mut painter = make_canvas_painter();
        let input = make_canvas_input(400.0, 300.0);
        let rect = WorldRect::from_xywh(5000.0, 5000.0, 100.0, 100.0);
        let style = DataItemStyle::default();
        let desc = CanvasDataItemDescriptor {
            title: Some("Far away"),
            supporting_text: Some("Still rendered"),
            metadata: None,
            style: &style,
        };

        let result = canvas_data_item(&mut painter, &input, rect, &desc);

        assert!(!result.hovered);
        assert!(!result.pressed);
        assert!(!result.clicked);
        assert_eq!(
            painter.quad_buffer.len(),
            1,
            "quad submitted regardless of visibility"
        );
        assert_eq!(
            painter.text_buffer.len(),
            2,
            "text submitted regardless of visibility"
        );
    }

    #[test]
    fn canvas_data_item_hover_changes_fill() {
        let style = DataItemStyle::default();
        let rect = WorldRect::from_xywh(-50.0, -50.0, 100.0, 100.0);

        let mut painter_normal = make_canvas_painter();
        let input_normal = make_canvas_input(100.0, 100.0);
        let desc = CanvasDataItemDescriptor {
            title: None,
            supporting_text: None,
            metadata: None,
            style: &style,
        };
        canvas_data_item(&mut painter_normal, &input_normal, rect, &desc);
        let fill_normal = painter_normal.quad_buffer[0].fill;

        let mut painter_hover = make_canvas_painter();
        let input_hover = make_canvas_input(400.0, 300.0);
        let desc = CanvasDataItemDescriptor {
            title: None,
            supporting_text: None,
            metadata: None,
            style: &style,
        };
        canvas_data_item(&mut painter_hover, &input_hover, rect, &desc);
        let fill_hover = painter_hover.quad_buffer[0].fill;

        assert_ne!(fill_normal, fill_hover, "hover should change fill color");
    }
}
