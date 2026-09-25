use std::path::PathBuf;

use akar_core::{AkarCore, InputState};
use akar_layout::{Layout, NodeId};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct FileDropResponse {
    pub hovered: bool,
    pub dropped_paths: Vec<PathBuf>,
}

/// Checks an existing node as a file target without creating layout or drawing.
/// The first eligible target checked claims each hover and drop; check specific
/// children before a parent fallback. Rechecking a target cannot claim twice.
pub fn file_drop_target(core: &mut AkarCore, layout: &Layout, node: NodeId) -> FileDropResponse {
    file_drop_target_input(
        &mut core.input,
        layout,
        node,
        core.draw_list.active_scissor(),
        core.draw_list.scale_factor(),
    )
}

fn file_drop_target_input(
    input: &mut InputState,
    layout: &Layout,
    node: NodeId,
    scissor: Option<[f32; 4]>,
    scale: f32,
) -> FileDropResponse {
    let Some(rect) = layout.try_rect(node).filter(|rect| valid_rect(*rect)) else {
        return FileDropResponse::default();
    };
    let hit = |position: [f32; 2]| {
        contains(rect, position)
            && scissor.is_none_or(|clip| {
                scale.is_finite()
                    && scale > 0.0
                    && valid_rect(clip)
                    && contains(clip, [position[0] * scale, position[1] * scale])
            })
    };

    let hovered = input.file_drag_position.is_some_and(&hit);
    let hovered = input.claim_file_hover(hovered);
    let dropped_paths = input.claim_file_drops(hit);

    FileDropResponse {
        hovered,
        dropped_paths,
    }
}

fn valid_rect(rect: [f32; 4]) -> bool {
    rect.iter().all(|value| value.is_finite()) && rect[2] > 0.0 && rect[3] > 0.0
}

fn contains([x, y, w, h]: [f32; 4], [px, py]: [f32; 2]) -> bool {
    px >= x && px < x + w && py >= y && py < y + h
}

#[cfg(test)]
mod tests {
    use super::*;
    use akar_core::FileDragInput;
    use akar_layout::{length, Size, Style};

    fn layout_with_child() -> (Layout, NodeId, NodeId, NodeId) {
        let mut layout = Layout::new();
        let child = layout.new_leaf(Style {
            size: Size {
                width: length(40.0_f32),
                height: length(40.0_f32),
            },
            ..Default::default()
        });
        let zero = layout.new_leaf(Style::default());
        let parent = layout.new_with_children(
            Style {
                size: Size {
                    width: length(100.0_f32),
                    height: length(100.0_f32),
                },
                ..Default::default()
            },
            &[child, zero],
        );
        layout.compute(parent, (Some(100.0), Some(100.0)), |_, _, _, _, _| {
            Size::ZERO
        });
        (layout, parent, child, zero)
    }

    fn target(input: &mut InputState, layout: &Layout, node: NodeId) -> FileDropResponse {
        file_drop_target_input(input, layout, node, None, 1.0)
    }

    fn drop_at(input: &mut InputState, position: [f32; 2], name: &str) {
        input.push_file_drag(FileDragInput::Drop {
            position,
            paths: vec![PathBuf::from(name)],
        });
    }

    #[test]
    fn first_checked_eligible_target_claims_parent_or_child() {
        let (layout, parent, child, _) = layout_with_child();
        let mut input = InputState::new();
        drop_at(&mut input, [10.0, 10.0], "child.txt");
        assert_eq!(
            target(&mut input, &layout, child).dropped_paths,
            [PathBuf::from("child.txt")]
        );
        assert!(target(&mut input, &layout, parent).dropped_paths.is_empty());

        input.begin_frame();
        drop_at(&mut input, [10.0, 10.0], "parent.txt");
        assert_eq!(
            target(&mut input, &layout, parent).dropped_paths,
            [PathBuf::from("parent.txt")]
        );
        assert!(target(&mut input, &layout, child).dropped_paths.is_empty());
    }

    #[test]
    fn ordinary_child_does_not_block_parent_and_repeated_checks_do_not_repeat() {
        let (layout, parent, _, _) = layout_with_child();
        let mut input = InputState::new();
        input.push_file_drag(FileDragInput::Enter {
            position: [10.0, 10.0],
            paths: vec![PathBuf::from("a")],
        });
        drop_at(&mut input, [10.0, 10.0], "a");
        drop_at(&mut input, [90.0, 90.0], "b");
        let response = target(&mut input, &layout, parent);
        assert_eq!(
            response.dropped_paths,
            [PathBuf::from("a"), PathBuf::from("b")]
        );
        assert_eq!(
            target(&mut input, &layout, parent),
            FileDropResponse::default()
        );
    }

    #[test]
    fn hover_is_claimed_once_and_uses_drag_position() {
        let (layout, parent, child, _) = layout_with_child();
        let mut input = InputState::new();
        input.set_mouse_pos(90.0, 90.0);
        input.push_file_drag(FileDragInput::Enter {
            position: [10.0, 10.0],
            paths: vec![],
        });
        assert!(target(&mut input, &layout, child).hovered);
        assert!(!target(&mut input, &layout, parent).hovered);
        assert!(!target(&mut input, &layout, child).hovered);
    }

    #[test]
    fn zero_area_and_nonfinite_rectangles_do_not_claim() {
        let (mut layout, parent, _, zero) = layout_with_child();
        let mut input = InputState::new();
        drop_at(&mut input, [0.0, 40.0], "zero");
        assert_eq!(
            target(&mut input, &layout, zero),
            FileDropResponse::default()
        );
        layout.set_screen_origin([f32::NAN, 0.0]);
        assert_eq!(
            target(&mut input, &layout, parent),
            FileDropResponse::default()
        );
    }

    #[test]
    fn scissor_uses_physical_coordinates_at_nonunit_scale() {
        let (layout, parent, _, _) = layout_with_child();
        let mut input = InputState::new();
        let scissor = [40.0, 40.0, 60.0, 60.0];
        drop_at(&mut input, [10.0, 10.0], "outside");
        drop_at(&mut input, [25.0, 25.0], "inside");
        assert_eq!(
            file_drop_target_input(&mut input, &layout, parent, Some(scissor), 2.0).dropped_paths,
            [PathBuf::from("inside")]
        );
        assert_eq!(
            target(&mut input, &layout, parent).dropped_paths,
            [PathBuf::from("outside")]
        );
    }
}
