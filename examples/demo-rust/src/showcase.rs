use akar_layout::{length, AlignItems, Display, FlexDirection, Layout, NodeId, Size, Style};

#[allow(dead_code)] // scaffolding for Tasks 3-7
pub struct ShowcaseLayout {
    pub root: NodeId,
    pub items: Vec<ShowcaseItem>,
}

#[allow(dead_code)] // scaffolding for Tasks 3-7
pub struct ShowcaseItem {
    pub node: NodeId,
    pub label: String,
}

#[allow(dead_code)] // scaffolding for Tasks 3-7
pub fn showcase_row(
    layout: &mut Layout,
    parent: NodeId,
    item_width: f32,
    item_height: f32,
    labels: &[&str],
    gap: f32,
) -> ShowcaseLayout {
    layout.set_style(
        parent,
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: Some(AlignItems::CENTER),
            gap: akar_layout::Size {
                width: length(gap),
                height: length(0.0_f32),
            },
            ..Default::default()
        },
    );

    let mut items = Vec::new();
    for &label in labels {
        let node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(item_width),
                height: length(item_height),
            },
            ..Default::default()
        });
        layout.add_child(parent, node);
        layout.register_label(label, node);
        items.push(ShowcaseItem {
            node,
            label: label.to_string(),
        });
    }

    ShowcaseLayout {
        root: parent,
        items,
    }
}

#[allow(dead_code)] // scaffolding for Tasks 3-7
pub fn showcase_grid(
    layout: &mut Layout,
    parent: NodeId,
    item_width: f32,
    item_height: f32,
    labels: &[&str],
    cols: usize,
    gap: f32,
) -> ShowcaseLayout {
    layout.set_style(
        parent,
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            gap: akar_layout::Size {
                width: length(0.0_f32),
                height: length(gap),
            },
            ..Default::default()
        },
    );

    let mut items = Vec::new();
    for row_labels in labels.chunks(cols) {
        let row = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: Some(AlignItems::CENTER),
            gap: akar_layout::Size {
                width: length(gap),
                height: length(0.0_f32),
            },
            ..Default::default()
        });
        layout.add_child(parent, row);
        for &label in row_labels {
            let node = layout.new_leaf(Style {
                flex_shrink: 0.0,
                size: Size {
                    width: length(item_width),
                    height: length(item_height),
                },
                ..Default::default()
            });
            layout.add_child(row, node);
            layout.register_label(label, node);
            items.push(ShowcaseItem {
                node,
                label: label.to_string(),
            });
        }
    }

    ShowcaseLayout {
        root: parent,
        items,
    }
}

#[allow(dead_code)] // scaffolding for Tasks 3-7
pub fn showcase_stack(
    layout: &mut Layout,
    parent: NodeId,
    item_width: f32,
    item_height: f32,
    labels: &[&str],
    gap: f32,
) -> ShowcaseLayout {
    layout.set_style(
        parent,
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            gap: akar_layout::Size {
                width: length(0.0_f32),
                height: length(gap),
            },
            ..Default::default()
        },
    );

    let mut items = Vec::new();
    for &label in labels {
        let node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(item_width),
                height: length(item_height),
            },
            ..Default::default()
        });
        layout.add_child(parent, node);
        layout.register_label(label, node);
        items.push(ShowcaseItem {
            node,
            label: label.to_string(),
        });
    }

    ShowcaseLayout {
        root: parent,
        items,
    }
}

#[allow(dead_code)] // scaffolding for Tasks 3-7
pub fn showcase_mini_viewport(
    layout: &mut Layout,
    parent: NodeId,
    width: f32,
    height: f32,
    label: &str,
) -> ShowcaseLayout {
    layout.set_style(
        parent,
        Style {
            display: Display::Flex,
            size: Size {
                width: length(width),
                height: length(height),
            },
            ..Default::default()
        },
    );

    layout.register_label(label, parent);

    ShowcaseLayout {
        root: parent,
        items: vec![ShowcaseItem {
            node: parent,
            label: label.to_string(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn showcase_row_creates_nodes() {
        let mut layout = Layout::new();
        let parent = layout.new_leaf(Style::default());
        let sl = showcase_row(&mut layout, parent, 100.0, 32.0, &["a", "b", "c"], 8.0);
        assert_eq!(sl.items.len(), 3);
        assert_eq!(sl.items[0].label, "a");
        assert_eq!(sl.items[1].label, "b");
        assert_eq!(sl.items[2].label, "c");
    }

    #[test]
    fn showcase_grid_wraps_rows() {
        let mut layout = Layout::new();
        let parent = layout.new_leaf(Style::default());
        let labels = ["a", "b", "c", "d", "e"];
        let sl = showcase_grid(&mut layout, parent, 100.0, 32.0, &labels, 2, 8.0);
        assert_eq!(sl.items.len(), 5);
    }

    #[test]
    fn showcase_stack_creates_nodes() {
        let mut layout = Layout::new();
        let parent = layout.new_leaf(Style::default());
        let sl = showcase_stack(&mut layout, parent, 200.0, 40.0, &["x", "y"], 4.0);
        assert_eq!(sl.items.len(), 2);
    }

    #[test]
    fn showcase_mini_viewport_single_item() {
        let mut layout = Layout::new();
        let parent = layout.new_leaf(Style::default());
        let sl = showcase_mini_viewport(&mut layout, parent, 300.0, 200.0, "overlay");
        assert_eq!(sl.items.len(), 1);
        assert_eq!(sl.items[0].label, "overlay");
    }
}
