use std::collections::HashMap;

pub use taffy::prelude::*;

mod responsive;
pub use responsive::responsive_columns;

mod rect;
pub use rect::WorldRect;

mod canvas_transform;
pub use canvas_transform::{
    compute_visible_world_rect, make_screen_to_world, make_world_to_screen, CanvasTransform,
};

pub type NodeId = taffy::NodeId;

pub struct AkarNodeContext {
    pub text_buffer_id: u64,
}

pub struct Layout {
    tree: TaffyTree<AkarNodeContext>,
    parents: HashMap<NodeId, NodeId>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            tree: TaffyTree::new(),
            parents: HashMap::new(),
        }
    }

    pub fn new_leaf(&mut self, style: Style) -> NodeId {
        self.tree.new_leaf(style).unwrap()
    }

    pub fn new_leaf_with_context(&mut self, style: Style, ctx: AkarNodeContext) -> NodeId {
        self.tree.new_leaf_with_context(style, ctx).unwrap()
    }

    pub fn new_with_children(&mut self, style: Style, children: &[NodeId]) -> NodeId {
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
    }

    pub fn remove(&mut self, node: NodeId) {
        self.parents.remove(&node);
        self.tree.remove(node).unwrap();
    }

    pub fn set_style(&mut self, node: NodeId, style: Style) {
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
        [x, y, l.size.width, l.size.height]
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
}
