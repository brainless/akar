pub use taffy::prelude::*;

pub type NodeId = taffy::NodeId;

pub struct AkarNodeContext {
    pub text_buffer_id: u64,
}

pub struct Layout {
    tree: TaffyTree<AkarNodeContext>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            tree: TaffyTree::new(),
        }
    }

    pub fn new_leaf(&mut self, style: Style) -> NodeId {
        self.tree.new_leaf(style).unwrap()
    }

    pub fn new_leaf_with_context(&mut self, style: Style, ctx: AkarNodeContext) -> NodeId {
        self.tree.new_leaf_with_context(style, ctx).unwrap()
    }

    pub fn new_with_children(&mut self, style: Style, children: &[NodeId]) -> NodeId {
        self.tree.new_with_children(style, children).unwrap()
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        self.tree.add_child(parent, child).unwrap();
    }

    pub fn set_children(&mut self, parent: NodeId, children: &[NodeId]) {
        self.tree.set_children(parent, children).unwrap();
    }

    pub fn remove(&mut self, node: NodeId) {
        self.tree.remove(node).unwrap();
    }

    pub fn set_style(&mut self, node: NodeId, style: Style) {
        self.tree.set_style(node, style).unwrap();
    }

    pub fn set_node_context(&mut self, node: NodeId, ctx: Option<AkarNodeContext>) {
        self.tree.set_node_context(node, ctx).unwrap();
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
            width: available.0.map(AvailableSpace::Definite).unwrap_or(AvailableSpace::MaxContent),
            height: available.1.map(AvailableSpace::Definite).unwrap_or(AvailableSpace::MaxContent),
        };
        self.tree
            .compute_layout_with_measure(root, available_space, measure_fn)
            .unwrap();
    }

    pub fn rect(&self, node: NodeId) -> [f32; 4] {
        let l = self.tree.layout(node).unwrap();
        [l.location.x, l.location.y, l.size.width, l.size.height]
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
}
