use std::collections::HashMap;

pub use taffy::prelude::*;
use taffy::style::Direction;

mod responsive;
pub use responsive::responsive_columns;

mod rect;
pub use rect::WorldRect;

mod canvas_transform;
pub use canvas_transform::{
    compute_visible_world_rect, make_screen_to_world, make_world_to_screen, CanvasTransform,
};

pub type NodeId = taffy::NodeId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AkarDirection {
    #[default]
    Ltr,
    Rtl,
}

impl From<AkarDirection> for Direction {
    fn from(direction: AkarDirection) -> Self {
        match direction {
            AkarDirection::Ltr => Direction::Ltr,
            AkarDirection::Rtl => Direction::Rtl,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AkarNodeContext {
    pub text_buffer_id: u64,
}

impl AkarNodeContext {
    pub const fn text(buffer_id: u64) -> Self {
        Self {
            text_buffer_id: buffer_id,
        }
    }

    pub const fn empty() -> Self {
        Self { text_buffer_id: 0 }
    }
}

pub struct Layout {
    tree: TaffyTree<AkarNodeContext>,
    parents: HashMap<NodeId, NodeId>,
    labels: HashMap<String, NodeId>,
    screen_origin: [f32; 2],
    namespace_id: u64,
    direction: AkarDirection,
    set_children_calls: u64,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            tree: TaffyTree::new(),
            parents: HashMap::new(),
            labels: HashMap::new(),
            screen_origin: [0.0; 2],
            namespace_id: 0,
            direction: AkarDirection::default(),
            set_children_calls: 0,
        }
    }

    pub fn set_screen_origin(&mut self, origin: [f32; 2]) {
        self.screen_origin = origin;
    }

    pub fn set_namespace_id(&mut self, id: u64) {
        self.namespace_id = id;
    }

    pub fn namespace_id(&self) -> u64 {
        self.namespace_id
    }

    pub fn set_direction(&mut self, direction: AkarDirection) {
        self.direction = direction;
    }

    pub fn direction(&self) -> AkarDirection {
        self.direction
    }

    pub fn widget_id(&self, node: NodeId) -> u64 {
        let local: u64 = node.into();
        if self.namespace_id == 0 {
            return local;
        }
        mix_widget_id(self.namespace_id ^ local.rotate_left(29)) & !(1 << 63)
    }

    pub fn widget_id_keyed(&self, node: NodeId, key: u64) -> u64 {
        let local: u64 = node.into();
        mix_widget_id(self.namespace_id ^ key ^ local.rotate_left(29)) & !(1 << 63)
    }

    pub fn register_label(&mut self, name: &str, node: NodeId) {
        self.labels.insert(name.to_string(), node);
    }

    pub fn resolve_label(&self, name: &str) -> Option<NodeId> {
        self.labels.get(name).copied()
    }

    pub fn labeled_rects(&self) -> Vec<(String, [f32; 4])> {
        self.labels
            .iter()
            .map(|(name, node)| (name.clone(), self.rect(*node)))
            .collect()
    }

    pub fn new_leaf(&mut self, mut style: Style) -> NodeId {
        style.direction = self.direction.into();
        self.tree.new_leaf(style).unwrap()
    }

    pub fn new_leaf_with_context(&mut self, mut style: Style, ctx: AkarNodeContext) -> NodeId {
        style.direction = self.direction.into();
        self.tree.new_leaf_with_context(style, ctx).unwrap()
    }

    pub fn new_with_children(&mut self, mut style: Style, children: &[NodeId]) -> NodeId {
        style.direction = self.direction.into();
        let node = self.tree.new_with_children(style, children).unwrap();
        for &child in children {
            self.parents.insert(child, node);
        }
        node
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        self.tree.add_child(parent, child).unwrap();
        self.parents.insert(child, parent);
    }

    pub fn set_children(&mut self, parent: NodeId, children: &[NodeId]) {
        self.tree.set_children(parent, children).unwrap();
        for &child in children {
            self.parents.insert(child, parent);
        }
        self.set_children_calls += 1;
    }

    pub fn children(&self, node: NodeId) -> Vec<NodeId> {
        self.tree.children(node).unwrap_or_default()
    }

    /// Number of times [`Layout::set_children`] has been called on this tree.
    /// Construct/compute/paint contract: tree relationships should be
    /// established once (or only on a deliberate layout rebuild), never once
    /// per redraw. Tests use this counter to catch a paint/redraw path that
    /// mutates child relationships every frame.
    pub fn set_children_calls(&self) -> u64 {
        self.set_children_calls
    }

    pub fn remove(&mut self, node: NodeId) {
        self.parents.remove(&node);
        self.tree.remove(node).unwrap();
    }

    pub fn set_style(&mut self, node: NodeId, mut style: Style) {
        style.direction = self.direction.into();
        self.tree.set_style(node, style).unwrap();
    }

    pub fn set_node_context(&mut self, node: NodeId, ctx: Option<AkarNodeContext>) {
        self.tree.set_node_context(node, ctx).unwrap();
    }

    pub fn set_padding(&mut self, node: NodeId, top: f32, right: f32, bottom: f32, left: f32) {
        let mut style = self.tree.style(node).unwrap().clone();
        style.padding = taffy::geometry::Rect {
            top: length(top),
            right: length(right),
            bottom: length(bottom),
            left: length(left),
        };
        self.tree.set_style(node, style).unwrap();
    }

    pub fn set_margin(&mut self, node: NodeId, top: f32, right: f32, bottom: f32, left: f32) {
        let mut style = self.tree.style(node).unwrap().clone();
        style.margin = taffy::geometry::Rect {
            top: length(top),
            right: length(right),
            bottom: length(bottom),
            left: length(left),
        };
        self.tree.set_style(node, style).unwrap();
    }

    pub fn compute<F>(&mut self, root: NodeId, available: (Option<f32>, Option<f32>), measure_fn: F)
    where
        F: FnMut(
            Size<Option<f32>>,
            Size<AvailableSpace>,
            NodeId,
            Option<&mut AkarNodeContext>,
            &Style,
        ) -> Size<f32>,
    {
        let available_space = Size {
            width: available
                .0
                .map(AvailableSpace::Definite)
                .unwrap_or(AvailableSpace::MaxContent),
            height: available
                .1
                .map(AvailableSpace::Definite)
                .unwrap_or(AvailableSpace::MaxContent),
        };
        self.tree
            .compute_layout_with_measure(root, available_space, measure_fn)
            .unwrap();
    }

    /// Compute layout using the `TextPipeline` for intrinsic text measurement.
    ///
    /// Text-bearing leaves identify themselves through
    /// `AkarNodeContext::text_buffer_id` (non-zero). The buffer must already
    /// exist in `text_pipeline`; the same buffer is reused by the paint path
    /// so measurement and rendering share the same shaped geometry.
    pub fn compute_with_text(
        &mut self,
        root: NodeId,
        available: (Option<f32>, Option<f32>),
        text_pipeline: &mut akar_core::TextPipeline,
    ) {
        self.compute(root, available, default_measure_fn(text_pipeline));
    }

    pub fn rect_offset(&self, node: NodeId, origin: [f32; 2]) -> [f32; 4] {
        let [x, y, w, h] = self.rect(node);
        [origin[0] + x, origin[1] + y, w, h]
    }

    pub fn rect(&self, node: NodeId) -> [f32; 4] {
        let l = self.tree.layout(node).unwrap();
        let mut x = l.location.x;
        let mut y = l.location.y;
        let mut current = node;
        while let Some(&parent) = self.parents.get(&current) {
            let pl = self.tree.layout(parent).unwrap();
            x += pl.location.x;
            y += pl.location.y;
            current = parent;
        }
        [
            self.screen_origin[0] + x,
            self.screen_origin[1] + y,
            l.size.width,
            l.size.height,
        ]
    }
}

fn mix_widget_id(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

/// Returns a measure closure for `Layout::compute` that resolves intrinsic
/// sizes from `TextPipeline`.
///
/// A node contributes a text-measured size when its `AkarNodeContext` holds a
/// non-zero `text_buffer_id` that already exists in `text_pipeline`. Nodes
/// without a context (or with `text_buffer_id == 0`) return `Size::ZERO`,
/// matching the historical default for non-text leaves.
///
/// Known dimensions from the parent take precedence. The available width
/// passed to `TextPipeline::measure_with_metadata` follows Taffy's
/// `AvailableSpace` semantics: `Definite` yields the exact width, `MinContent`
/// constrains to `0.0` (forcing wrap at every break opportunity), and
/// `MaxContent` leaves the buffer unconstrained.
#[allow(clippy::type_complexity)]
pub fn default_measure_fn<'a>(
    text_pipeline: &'a mut akar_core::TextPipeline,
) -> impl FnMut(
    Size<Option<f32>>,
    Size<AvailableSpace>,
    NodeId,
    Option<&mut AkarNodeContext>,
    &Style,
) -> Size<f32>
       + 'a {
    move |known_dimensions, available_space, _node, context, _style| {
        let Some(ctx) = context else {
            return Size::ZERO;
        };
        if ctx.text_buffer_id == 0 {
            return Size::ZERO;
        }

        let result = text_pipeline.measure_with_metadata(
            ctx.text_buffer_id,
            akar_core::TextMeasureInput {
                known_width: known_dimensions.width,
                known_height: known_dimensions.height,
                available_width: match available_space.width {
                    AvailableSpace::Definite(w) => Some(w.max(0.0)),
                    AvailableSpace::MinContent => Some(0.0),
                    AvailableSpace::MaxContent => None,
                },
            },
        );

        Size {
            width: result.width,
            height: result.height,
        }
    }
}

pub struct TwoColumnLayout {
    pub left: NodeId,
    pub separator: NodeId,
    pub right: NodeId,
}

pub struct ThreeColumnLayout {
    pub left: NodeId,
    pub sep_left: NodeId,
    pub middle: NodeId,
    pub sep_right: NodeId,
    pub right: NodeId,
}

impl Layout {
    pub fn two_column(
        &mut self,
        parent: NodeId,
        left_fraction: f32,
        separator_thickness: f32,
    ) -> TwoColumnLayout {
        self.set_style(
            parent,
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                size: Size {
                    width: Dimension::percent(1.0),
                    height: Dimension::percent(1.0),
                },
                ..Default::default()
            },
        );

        let left_fraction = left_fraction.clamp(0.0, 1.0);
        let right_fraction = 1.0 - left_fraction;

        let left = self.new_leaf(Style {
            flex_grow: left_fraction,
            flex_shrink: 1.0,
            ..Default::default()
        });
        let separator = self.new_leaf(Style {
            flex_grow: 0.0,
            flex_shrink: 0.0,
            size: Size {
                width: length(separator_thickness),
                height: Dimension::auto(),
            },
            ..Default::default()
        });
        let right = self.new_leaf(Style {
            flex_grow: right_fraction,
            flex_shrink: 1.0,
            ..Default::default()
        });

        self.set_children(parent, &[left, separator, right]);

        TwoColumnLayout {
            left,
            separator,
            right,
        }
    }

    pub fn three_column(
        &mut self,
        parent: NodeId,
        fractions: [f32; 3],
        separator_thickness: f32,
    ) -> ThreeColumnLayout {
        self.set_style(
            parent,
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                size: Size {
                    width: Dimension::percent(1.0),
                    height: Dimension::percent(1.0),
                },
                ..Default::default()
            },
        );

        let left = self.new_leaf(Style {
            flex_grow: fractions[0],
            flex_shrink: 1.0,
            ..Default::default()
        });
        let sep_left = self.new_leaf(Style {
            flex_grow: 0.0,
            flex_shrink: 0.0,
            size: Size {
                width: length(separator_thickness),
                height: Dimension::auto(),
            },
            ..Default::default()
        });
        let middle = self.new_leaf(Style {
            flex_grow: fractions[1],
            flex_shrink: 1.0,
            ..Default::default()
        });
        let sep_right = self.new_leaf(Style {
            flex_grow: 0.0,
            flex_shrink: 0.0,
            size: Size {
                width: length(separator_thickness),
                height: Dimension::auto(),
            },
            ..Default::default()
        });
        let right = self.new_leaf(Style {
            flex_grow: fractions[2],
            flex_shrink: 1.0,
            ..Default::default()
        });

        self.set_children(parent, &[left, sep_left, middle, sep_right, right]);

        ThreeColumnLayout {
            left,
            sep_left,
            middle,
            sep_right,
            right,
        }
    }
}

pub struct PageConfig {
    pub header_height: Option<f32>,
    pub footer_height: Option<f32>,
    pub sidebar_left_width: Option<f32>,
    pub sidebar_right_width: Option<f32>,
}

pub struct PageLayout {
    pub root: NodeId,
    pub header: Option<NodeId>,
    pub body: NodeId,
    pub sidebar_left: Option<NodeId>,
    pub main: NodeId,
    pub sidebar_right: Option<NodeId>,
    pub footer: Option<NodeId>,
}

impl Layout {
    pub fn page(&mut self, config: PageConfig) -> PageLayout {
        let header = config.header_height.map(|h| {
            self.new_leaf(Style {
                flex_shrink: 0.0,
                size: Size {
                    width: Dimension::percent(1.0),
                    height: length(h),
                },
                ..Default::default()
            })
        });

        let footer = config.footer_height.map(|h| {
            self.new_leaf(Style {
                flex_shrink: 0.0,
                size: Size {
                    width: Dimension::percent(1.0),
                    height: length(h),
                },
                ..Default::default()
            })
        });

        let sidebar_left = config.sidebar_left_width.map(|w| {
            self.new_leaf(Style {
                flex_shrink: 0.0,
                size: Size {
                    width: length(w),
                    height: Dimension::percent(1.0),
                },
                ..Default::default()
            })
        });

        let sidebar_right = config.sidebar_right_width.map(|w| {
            self.new_leaf(Style {
                flex_shrink: 0.0,
                size: Size {
                    width: length(w),
                    height: Dimension::percent(1.0),
                },
                ..Default::default()
            })
        });

        let main = self.new_leaf(Style {
            flex_grow: 1.0,
            size: Size {
                width: Dimension::auto(),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        });

        let mut body_children: Vec<NodeId> = Vec::new();
        if let Some(sl) = sidebar_left {
            body_children.push(sl);
        }
        body_children.push(main);
        if let Some(sr) = sidebar_right {
            body_children.push(sr);
        }

        let body = self.new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_grow: 1.0,
                size: Size {
                    width: Dimension::percent(1.0),
                    height: Dimension::auto(),
                },
                ..Default::default()
            },
            &body_children,
        );

        let mut root_children: Vec<NodeId> = Vec::new();
        if let Some(h) = header {
            root_children.push(h);
        }
        root_children.push(body);
        if let Some(f) = footer {
            root_children.push(f);
        }

        let root = self.new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: Dimension::percent(1.0),
                    height: Dimension::percent(1.0),
                },
                ..Default::default()
            },
            &root_children,
        );

        PageLayout {
            root,
            header,
            body,
            sidebar_left,
            main,
            sidebar_right,
            footer,
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flex_container_with_two_children() {
        let mut layout = Layout::new();

        let child_a = layout.new_leaf(Style {
            display: Display::Flex,
            size: Size {
                width: length(100.0),
                height: length(50.0),
            },
            ..Default::default()
        });

        let child_b = layout.new_leaf(Style {
            display: Display::Flex,
            size: Size {
                width: length(100.0),
                height: length(50.0),
            },
            ..Default::default()
        });

        let root = layout.new_with_children(
            Style {
                display: Display::Flex,
                ..Default::default()
            },
            &[child_a, child_b],
        );

        layout.compute(root, (Some(400.0), Some(300.0)), |_, _, _, _, _| Size::ZERO);

        let r = layout.rect(child_a);
        assert_eq!(r[0], 0.0, "child_a.x should be 0.0");
        assert_eq!(r[2], 100.0, "child_a.width should be 100.0");

        let r_b = layout.rect(child_b);
        assert_eq!(r_b[0], 100.0, "child_b.x should be 100.0");
        assert_eq!(r_b[2], 100.0, "child_b.width should be 100.0");
    }

    #[test]
    fn direction_defaults_to_ltr_and_round_trips() {
        let mut layout = Layout::new();
        assert_eq!(layout.direction(), AkarDirection::Ltr);

        layout.set_direction(AkarDirection::Rtl);
        assert_eq!(layout.direction(), AkarDirection::Rtl);
    }

    #[test]
    fn direction_stamp_reverses_flex_row_main_axis() {
        // LTR: child_a first (left), child_b second (right).
        let mut ltr = Layout::new();

        let child_a = ltr.new_leaf(Style {
            display: Display::Flex,
            size: Size {
                width: length(100.0),
                height: length(50.0),
            },
            ..Default::default()
        });

        let child_b = ltr.new_leaf(Style {
            display: Display::Flex,
            size: Size {
                width: length(100.0),
                height: length(50.0),
            },
            ..Default::default()
        });

        let root = ltr.new_with_children(
            Style {
                display: Display::Flex,
                ..Default::default()
            },
            &[child_a, child_b],
        );

        ltr.compute(root, (Some(400.0), Some(300.0)), |_, _, _, _, _| Size::ZERO);

        let ltr_a = ltr.rect(child_a);
        let ltr_b = ltr.rect(child_b);
        assert_eq!(ltr_a[0], 0.0, "LTR child_a.x should be 0.0");
        assert_eq!(ltr_b[0], 100.0, "LTR child_b.x should be 100.0");

        // RTL: a single layout.set_direction(Rtl) call before construction should
        // reverse the main axis for every node created afterward, without any
        // caller touching Style.direction directly.
        let mut rtl = Layout::new();
        rtl.set_direction(AkarDirection::Rtl);

        let child_a = rtl.new_leaf(Style {
            display: Display::Flex,
            size: Size {
                width: length(100.0),
                height: length(50.0),
            },
            ..Default::default()
        });

        let child_b = rtl.new_leaf(Style {
            display: Display::Flex,
            size: Size {
                width: length(100.0),
                height: length(50.0),
            },
            ..Default::default()
        });

        let root = rtl.new_with_children(
            Style {
                display: Display::Flex,
                ..Default::default()
            },
            &[child_a, child_b],
        );

        rtl.compute(root, (Some(400.0), Some(300.0)), |_, _, _, _, _| Size::ZERO);

        let rtl_a = rtl.rect(child_a);
        let rtl_b = rtl.rect(child_b);
        assert_eq!(rtl_a[0], 300.0, "RTL child_a.x should be 300.0");
        assert_eq!(rtl_b[0], 200.0, "RTL child_b.x should be 200.0");
        assert!(
            rtl_a[0] > rtl_b[0],
            "RTL should place the first child to the right of the second"
        );
    }

    #[test]
    fn two_column_equal_split() {
        let mut layout = Layout::new();
        let root = layout.new_leaf(Style::default());
        let cols = layout.two_column(root, 0.5, 1.0);
        layout.compute(root, (Some(401.0), Some(300.0)), |_, _, _, _, _| Size::ZERO);

        let left_r = layout.rect(cols.left);
        let sep_r = layout.rect(cols.separator);
        let right_r = layout.rect(cols.right);

        assert_eq!(sep_r[2], 1.0, "separator width should be 1.0");
        assert!(
            (left_r[2] - 200.0).abs() < 1.0,
            "left width should be ~200.0, got {}",
            left_r[2]
        );
        assert!(
            (right_r[2] - 200.0).abs() < 1.0,
            "right width should be ~200.0, got {}",
            right_r[2]
        );
    }

    #[test]
    fn two_column_30_70_split() {
        let mut layout = Layout::new();
        let root = layout.new_leaf(Style::default());
        let cols = layout.two_column(root, 0.3, 2.0);
        layout.compute(root, (Some(402.0), Some(300.0)), |_, _, _, _, _| Size::ZERO);

        let left_r = layout.rect(cols.left);
        let sep_r = layout.rect(cols.separator);
        let right_r = layout.rect(cols.right);

        assert_eq!(sep_r[2], 2.0);
        assert!(
            (left_r[2] - 120.0).abs() < 1.0,
            "left width should be ~120.0, got {}",
            left_r[2]
        );
        assert!(
            (right_r[2] - 280.0).abs() < 1.0,
            "right width should be ~280.0, got {}",
            right_r[2]
        );
    }

    #[test]
    fn three_column_weighted_split() {
        let mut layout = Layout::new();
        let root = layout.new_leaf(Style::default());
        let cols = layout.three_column(root, [1.0, 2.0, 1.0], 1.0);
        layout.compute(root, (Some(402.0), Some(300.0)), |_, _, _, _, _| Size::ZERO);

        let left_r = layout.rect(cols.left);
        let middle_r = layout.rect(cols.middle);
        let right_r = layout.rect(cols.right);
        let sep_l = layout.rect(cols.sep_left);
        let sep_r = layout.rect(cols.sep_right);

        assert_eq!(sep_l[2], 1.0);
        assert_eq!(sep_r[2], 1.0);
        assert!(
            (left_r[2] - 100.0).abs() < 1.0,
            "left width should be ~100.0, got {}",
            left_r[2]
        );
        assert!(
            (middle_r[2] - 200.0).abs() < 1.0,
            "middle width should be ~200.0, got {}",
            middle_r[2]
        );
        assert!(
            (right_r[2] - 100.0).abs() < 1.0,
            "right width should be ~100.0, got {}",
            right_r[2]
        );
    }

    #[test]
    fn page_with_header_and_left_sidebar() {
        let mut layout = Layout::new();
        let page = layout.page(PageConfig {
            header_height: Some(60.0),
            footer_height: None,
            sidebar_left_width: Some(200.0),
            sidebar_right_width: None,
        });
        layout.compute(page.root, (Some(800.0), Some(600.0)), |_, _, _, _, _| {
            Size::ZERO
        });

        let header_r = layout.rect(page.header.unwrap());
        let sidebar_r = layout.rect(page.sidebar_left.unwrap());
        let main_r = layout.rect(page.main);

        assert_eq!(header_r[2], 800.0, "header width should be 800.0");
        assert_eq!(header_r[3], 60.0, "header height should be 60.0");
        assert_eq!(sidebar_r[2], 200.0, "sidebar width should be 200.0");
        assert_eq!(sidebar_r[3], 540.0, "sidebar height should be 540.0");
        assert!(
            (main_r[2] - 600.0).abs() < 1.0,
            "main width should be ~600.0, got {}",
            main_r[2]
        );
        assert_eq!(main_r[3], 540.0, "main height should be 540.0");
    }

    #[test]
    fn page_minimal_no_optional_regions() {
        let mut layout = Layout::new();
        let page = layout.page(PageConfig {
            header_height: None,
            footer_height: None,
            sidebar_left_width: None,
            sidebar_right_width: None,
        });
        layout.compute(page.root, (Some(800.0), Some(600.0)), |_, _, _, _, _| {
            Size::ZERO
        });

        assert!(page.header.is_none());
        assert!(page.footer.is_none());
        assert!(page.sidebar_left.is_none());
        assert!(page.sidebar_right.is_none());

        let main_r = layout.rect(page.main);
        assert_eq!(main_r[2], 800.0, "main width should be 800.0");
        assert_eq!(main_r[3], 600.0, "main height should be 600.0");
    }

    #[test]
    fn set_padding_affects_child_position() {
        let mut layout = Layout::new();
        let child = layout.new_leaf(Style {
            size: Size {
                width: length(50.0),
                height: length(50.0),
            },
            ..Default::default()
        });
        let root = layout.new_with_children(
            Style {
                display: Display::Flex,
                size: Size {
                    width: length(200.0),
                    height: length(200.0),
                },
                ..Default::default()
            },
            &[child],
        );
        layout.set_padding(root, 20.0, 20.0, 20.0, 20.0);
        layout.compute(root, (Some(200.0), Some(200.0)), |_, _, _, _, _| Size::ZERO);

        let r = layout.rect(child);
        assert!((r[0] - 20.0).abs() < 1.0, "child.x = {}", r[0]);
        assert!((r[1] - 20.0).abs() < 1.0, "child.y = {}", r[1]);
    }

    #[test]
    fn rect_offset_shifts_by_origin() {
        let mut layout = Layout::new();
        let child = layout.new_leaf(Style {
            size: Size {
                width: length(40.0),
                height: length(20.0),
            },
            ..Default::default()
        });
        let root = layout.new_with_children(Style::default(), &[child]);
        layout.compute(root, (Some(200.0), Some(200.0)), |_, _, _, _, _| Size::ZERO);

        let r = layout.rect_offset(child, [100.0, 50.0]);
        assert_eq!(r[0], 100.0);
        assert_eq!(r[1], 50.0);
        assert_eq!(r[2], 40.0);
        assert_eq!(r[3], 20.0);
    }

    #[test]
    fn set_margin_pushes_node() {
        let mut layout = Layout::new();
        let child = layout.new_leaf(Style {
            size: Size {
                width: length(50.0),
                height: length(50.0),
            },
            ..Default::default()
        });
        let root = layout.new_with_children(
            Style {
                display: Display::Flex,
                ..Default::default()
            },
            &[child],
        );
        layout.set_margin(child, 10.0, 0.0, 0.0, 15.0);
        layout.compute(root, (Some(200.0), Some(200.0)), |_, _, _, _, _| Size::ZERO);

        let r = layout.rect(child);
        assert!((r[0] - 15.0).abs() < 1.0, "child.x = {}", r[0]);
        assert!((r[1] - 10.0).abs() < 1.0, "child.y = {}", r[1]);
    }

    #[test]
    fn register_and_resolve_label() {
        let mut layout = Layout::new();
        let child = layout.new_leaf(Style {
            size: Size {
                width: length(60.0),
                height: length(60.0),
            },
            ..Default::default()
        });
        let root = layout.new_with_children(Style::default(), &[child]);
        layout.compute(root, (Some(200.0), Some(200.0)), |_, _, _, _, _| Size::ZERO);

        layout.register_label("my_box", child);
        assert_eq!(layout.resolve_label("my_box"), Some(child));
        assert_eq!(layout.resolve_label("missing"), None);

        layout.register_label("my_box", root);
        assert_eq!(layout.resolve_label("my_box"), Some(root));
    }

    #[test]
    fn labeled_rects_resolves_registered_nodes() {
        let mut layout = Layout::new();
        let child = layout.new_leaf(Style {
            size: Size {
                width: length(60.0),
                height: length(60.0),
            },
            ..Default::default()
        });
        let root = layout.new_with_children(
            Style {
                display: Display::Flex,
                ..Default::default()
            },
            &[child],
        );
        layout.set_padding(root, 10.0, 0.0, 0.0, 10.0);
        layout.compute(root, (Some(200.0), Some(200.0)), |_, _, _, _, _| Size::ZERO);

        layout.register_label("child", child);
        let rects = layout.labeled_rects();
        assert_eq!(rects.len(), 1);
        let (name, rect) = &rects[0];
        assert_eq!(name, "child");
        assert!((rect[2] - 60.0).abs() < 1.0);
        assert!((rect[3] - 60.0).abs() < 1.0);
    }

    #[test]
    fn portal_child_rects_offset_by_origin() {
        let mut layout = Layout::new();
        layout.set_screen_origin([100.0, 50.0]);

        let child = layout.new_leaf(Style {
            size: Size {
                width: length(40.0),
                height: length(20.0),
            },
            ..Default::default()
        });
        let root = layout.new_with_children(Style::default(), &[child]);
        layout.compute(root, (Some(200.0), Some(200.0)), |_, _, _, _, _| Size::ZERO);

        let r = layout.rect(child);
        assert_eq!(r[0], 100.0, "child.x should include screen_origin.x");
        assert_eq!(r[1], 50.0, "child.y should include screen_origin.y");
        assert_eq!(r[2], 40.0);
        assert_eq!(r[3], 20.0);
    }

    #[test]
    fn default_layout_origin_zero() {
        let mut layout = Layout::new();

        let child = layout.new_leaf(Style {
            size: Size {
                width: length(40.0),
                height: length(20.0),
            },
            ..Default::default()
        });
        let root = layout.new_with_children(Style::default(), &[child]);
        layout.compute(root, (Some(200.0), Some(200.0)), |_, _, _, _, _| Size::ZERO);

        let r = layout.rect(child);
        assert_eq!(r[0], 0.0);
        assert_eq!(r[1], 0.0);
        assert_eq!(r[2], 40.0);
        assert_eq!(r[3], 20.0);
    }

    #[test]
    fn portal_same_local_nodes_distinct_widget_ids() {
        let mut layout_a = Layout::new();
        layout_a.set_namespace_id(1000);
        let node_a = layout_a.new_leaf(Style::default());

        let mut layout_b = Layout::new();
        layout_b.set_namespace_id(2000);
        let node_b = layout_b.new_leaf(Style::default());

        let id_a = layout_a.widget_id(node_a);
        let id_b = layout_b.widget_id(node_b);

        assert_ne!(
            id_a, id_b,
            "different namespaces must produce distinct widget IDs"
        );
    }

    #[test]
    fn portal_widget_ids_do_not_overlap_when_local_nodes_differ() {
        let mut layout_a = Layout::new();
        layout_a.set_namespace_id(1);
        let _root_a = layout_a.new_leaf(Style::default());
        let input_a = layout_a.new_leaf(Style::default());

        let mut layout_b = Layout::new();
        layout_b.set_namespace_id(2);
        let _root_b = layout_b.new_leaf(Style::default());
        let button_b = layout_b.new_leaf(Style::default());

        assert_ne!(layout_a.widget_id(input_a), layout_b.widget_id(button_b));
    }

    #[test]
    fn layout_drop_recreate_same_namespace_same_widget_id() {
        let local_node_idx: u64;

        {
            let mut layout = Layout::new();
            layout.set_namespace_id(42);
            let node = layout.new_leaf(Style::default());
            local_node_idx = u64::from(node);
            let _ = layout.widget_id(node);
        }

        {
            let mut layout = Layout::new();
            layout.set_namespace_id(42);
            let node: NodeId = local_node_idx.into();
            let id = layout.widget_id(node);
            assert_eq!(
                id,
                mix_widget_id(42 ^ local_node_idx.rotate_left(29)) & !(1 << 63)
            );
        }
    }

    #[test]
    fn zero_area_portal_root() {
        let mut layout = Layout::new();
        layout.set_screen_origin([200.0, 100.0]);

        let root = layout.new_leaf(Style::default());
        layout.compute(root, (Some(0.0), Some(0.0)), |_, _, _, _, _| Size::ZERO);

        let r = layout.rect(root);
        assert_eq!(r[0], 200.0);
        assert_eq!(r[1], 100.0);
        assert_eq!(r[2], 0.0);
        assert_eq!(r[3], 0.0);
    }

    #[test]
    fn rect_offset_composes_with_screen_origin() {
        let mut layout = Layout::new();
        layout.set_screen_origin([100.0, 50.0]);

        let child = layout.new_leaf(Style {
            size: Size {
                width: length(40.0),
                height: length(20.0),
            },
            ..Default::default()
        });
        let root = layout.new_with_children(Style::default(), &[child]);
        layout.compute(root, (Some(200.0), Some(200.0)), |_, _, _, _, _| Size::ZERO);

        let r = layout.rect_offset(child, [10.0, 20.0]);
        assert_eq!(
            r[0], 110.0,
            "rect_offset adds its origin on top of screen_origin"
        );
        assert_eq!(r[1], 70.0);
        assert_eq!(r[2], 40.0);
        assert_eq!(r[3], 20.0);
    }

    #[test]
    fn portal_nested_children_offset_by_origin() {
        let mut layout = Layout::new();
        layout.set_screen_origin([200.0, 100.0]);

        let grandchild = layout.new_leaf(Style {
            size: Size {
                width: length(20.0),
                height: length(10.0),
            },
            ..Default::default()
        });
        let child = layout.new_with_children(
            Style {
                display: Display::Flex,
                size: Size {
                    width: length(100.0),
                    height: length(50.0),
                },
                padding: taffy::geometry::Rect {
                    top: length(5.0),
                    right: length(5.0),
                    bottom: length(5.0),
                    left: length(5.0),
                },
                ..Default::default()
            },
            &[grandchild],
        );
        let root = layout.new_with_children(Style::default(), &[child]);
        layout.compute(root, (Some(400.0), Some(300.0)), |_, _, _, _, _| Size::ZERO);

        let cr = layout.rect(child);
        assert_eq!(cr[0], 200.0, "child.x includes screen_origin");
        assert_eq!(cr[1], 100.0, "child.y includes screen_origin");

        let gr = layout.rect(grandchild);
        assert_eq!(gr[0], 205.0, "grandchild.x includes origin + padding");
        assert_eq!(gr[1], 105.0, "grandchild.y includes origin + padding");
    }

    #[test]
    fn portal_widget_id_deterministic_for_same_namespace() {
        let mut layout_a = Layout::new();
        layout_a.set_namespace_id(42);
        let node_a = layout_a.new_leaf(Style::default());
        let id_first = layout_a.widget_id(node_a);

        let mut layout_b = Layout::new();
        layout_b.set_namespace_id(42);
        let local_idx: u64 = node_a.into();
        let node_b: NodeId = local_idx.into();
        let id_second = layout_b.widget_id(node_b);

        assert_eq!(
            id_first, id_second,
            "same namespace + same local node = same widget ID"
        );
    }

    #[test]
    fn widget_id_keyed_different_keys_produce_different_ids() {
        let mut layout = Layout::new();
        layout.set_namespace_id(0);
        let node = layout.new_leaf(Style::default());

        let id_a = layout.widget_id_keyed(node, 1000);
        let id_b = layout.widget_id_keyed(node, 2000);

        assert_ne!(
            id_a, id_b,
            "different keys must produce different widget IDs"
        );
    }

    #[test]
    fn widget_id_keyed_same_key_produces_same_id() {
        let mut layout_a = Layout::new();
        layout_a.set_namespace_id(0);
        let node_a = layout_a.new_leaf(Style::default());
        let id_first = layout_a.widget_id_keyed(node_a, 42);

        let mut layout_b = Layout::new();
        layout_b.set_namespace_id(0);
        let local_idx: u64 = node_a.into();
        let node_b: NodeId = local_idx.into();
        let id_second = layout_b.widget_id_keyed(node_b, 42);

        assert_eq!(
            id_first, id_second,
            "same key + same local node = same widget ID"
        );
    }

    #[test]
    fn widget_id_keyed_composes_with_namespace() {
        let mut layout_a = Layout::new();
        layout_a.set_namespace_id(100);
        let node_a = layout_a.new_leaf(Style::default());
        let id_a = layout_a.widget_id_keyed(node_a, 42);

        let mut layout_b = Layout::new();
        layout_b.set_namespace_id(200);
        let local_idx: u64 = node_a.into();
        let node_b: NodeId = local_idx.into();
        let id_b = layout_b.widget_id_keyed(node_b, 42);

        assert_ne!(
            id_a, id_b,
            "different namespaces must produce different keyed IDs"
        );
    }

    #[test]
    fn widget_id_keyed_differs_from_plain_widget_id() {
        let mut layout = Layout::new();
        layout.set_namespace_id(0);
        let node = layout.new_leaf(Style::default());

        let plain = layout.widget_id(node);
        let keyed = layout.widget_id_keyed(node, 12345);

        assert_ne!(plain, keyed, "keyed ID should differ from plain widget ID");
    }

    #[test]
    fn widget_id_keyed_high_bit_cleared() {
        let mut layout = Layout::new();
        layout.set_namespace_id(u64::MAX);
        let node = layout.new_leaf(Style::default());

        let id = layout.widget_id_keyed(node, u64::MAX);

        assert_eq!(id >> 63, 0, "high bit must be cleared");
    }
}
