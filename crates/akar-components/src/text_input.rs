use akar_core::{AkarCore, CaretMotion, Key, QuadCall, TextCall, Z_TEXT_FOREGROUND};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::text_edit::{
    apply_targeted_pastes, clipboard_shortcut, collapse_selection_visually, delete_selection,
    next_boundary, normalize_paste, previous_boundary, replace_selection, TextEditState,
};
use crate::AkarTheme;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TextInputResponse {
    pub changed: bool,
    pub submitted: bool,
    pub copy_text: Option<String>,
    pub request_paste: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn text_input(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    value: &mut String,
    edit_state: &mut TextEditState,
    placeholder: &str,
    cursor_visible: bool,
    theme: &AkarTheme,
) -> TextInputResponse {
    text_input_masked(
        core,
        layout,
        node_id,
        value,
        edit_state,
        placeholder,
        cursor_visible,
        theme,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn text_input_masked(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    value: &mut String,
    edit_state: &mut TextEditState,
    placeholder: &str,
    cursor_visible: bool,
    theme: &AkarTheme,
    masked: bool,
) -> TextInputResponse {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return TextInputResponse::default();
    }

    let id_u64 = layout.widget_id(node_id);

    if core.input.is_clicked(rect) {
        core.input.focused_id = Some(id_u64);
    }

    if core.input.focused_id == Some(id_u64)
        && core.input.mouse_buttons_pressed[0]
        && !core.input.is_hovering(rect)
    {
        core.input.focused_id = None;
    }

    let focused = core.input.focused_id == Some(id_u64);

    let text_x = rect[0] + theme.padding_x;
    let text_y = rect[1] + theme.padding_y;
    let max_text_width = (rect[2] - 2.0 * theme.padding_x).max(0.0);
    let metrics = glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2);

    let mut changed = false;
    let mut submitted = false;
    let mut copy_text = None;
    let mut request_paste = false;

    if focused {
        edit_state.normalize(value);
        changed |= apply_targeted_pastes(
            &core.input,
            core.input.focused_id,
            id_u64,
            value,
            edit_state,
            false,
        );
        let chars: String = core.input.chars.iter().collect();
        if !chars.is_empty() {
            changed |= replace_selection(value, edit_state, &normalize_paste(&chars, false));
        }

        for event in core.input.key_events.clone() {
            if core.text_edit_keybindings.matches_select_all(&event) {
                edit_state.select_all(value);
                continue;
            }
            let clipboard =
                clipboard_shortcut(&core.text_edit_keybindings, &event, value, edit_state);
            if clipboard.2 {
                if clipboard.0.is_some() {
                    copy_text = clipboard.0;
                }
                request_paste |= clipboard.1;
                continue;
            }
            if event.key == Key::Backspace {
                if edit_state.has_selection() {
                    changed |= delete_selection(value, edit_state);
                } else if edit_state.cursor > 0 {
                    let start = previous_boundary(value, edit_state.cursor);
                    edit_state.anchor = start;
                    changed |= delete_selection(value, edit_state);
                }
            } else if event.key == Key::Delete {
                if edit_state.has_selection() {
                    changed |= delete_selection(value, edit_state);
                } else if edit_state.cursor < value.len() {
                    let end = next_boundary(value, edit_state.cursor);
                    edit_state.anchor = end;
                    changed |= delete_selection(value, edit_state);
                }
            } else {
                match event.key {
                    Key::Left if edit_state.has_selection() => {
                        let shaped = reshape_for_motion(
                            core,
                            id_u64,
                            value,
                            masked,
                            metrics,
                            max_text_width,
                        );
                        collapse_selection_for_motion(
                            &core.text_pipeline,
                            &shaped,
                            edit_state,
                            CaretMotion::Left,
                        );
                    }
                    Key::Left => {
                        let shaped = reshape_for_motion(
                            core,
                            id_u64,
                            value,
                            masked,
                            metrics,
                            max_text_width,
                        );
                        let display_offset = core.text_pipeline.move_caret(
                            shaped.buffer_id,
                            shaped.offsets.to_display(edit_state.cursor),
                            CaretMotion::Left,
                        );
                        edit_state.cursor = shaped.offsets.to_original(display_offset);
                        edit_state.anchor = edit_state.cursor;
                    }
                    Key::Right if edit_state.has_selection() => {
                        let shaped = reshape_for_motion(
                            core,
                            id_u64,
                            value,
                            masked,
                            metrics,
                            max_text_width,
                        );
                        collapse_selection_for_motion(
                            &core.text_pipeline,
                            &shaped,
                            edit_state,
                            CaretMotion::Right,
                        );
                    }
                    Key::Right => {
                        let shaped = reshape_for_motion(
                            core,
                            id_u64,
                            value,
                            masked,
                            metrics,
                            max_text_width,
                        );
                        let display_offset = core.text_pipeline.move_caret(
                            shaped.buffer_id,
                            shaped.offsets.to_display(edit_state.cursor),
                            CaretMotion::Right,
                        );
                        edit_state.cursor = shaped.offsets.to_original(display_offset);
                        edit_state.anchor = edit_state.cursor;
                    }
                    Key::Home => {
                        edit_state.cursor = 0;
                        edit_state.anchor = 0;
                    }
                    Key::End => {
                        edit_state.cursor = value.len();
                        edit_state.anchor = value.len();
                    }
                    Key::Enter => {
                        submitted = true;
                    }
                    Key::Escape => {
                        core.input.focused_id = None;
                    }
                    _ => {}
                }
            }
        }
    }

    if !focused {
        edit_state.normalize(value);
    }

    let border_color = if focused {
        theme.primary
    } else {
        theme.base_300
    };

    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(theme.base_200),
        border_color: color_to_f32(border_color),
        corner_radii: [theme.radius_field; 4],
        border_width: theme.border_width,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let display_text = if value.is_empty() && !focused {
        placeholder.to_string()
    } else if masked && !value.is_empty() {
        "*".repeat(value.chars().count())
    } else {
        value.clone()
    };
    let text_color = if value.is_empty() && !focused {
        theme.base_300
    } else {
        theme.base_content
    };

    let buffer_id = core.text_pipeline.set_text(
        Some(id_u64),
        &display_text,
        metrics,
        Some(max_text_width),
        None,
        None,
    );

    let offsets = TextOffsetMap::new(value, masked);
    let geometry = core.text_pipeline.geometry(
        buffer_id,
        &display_text,
        offsets.to_display(edit_state.cursor),
        offsets.to_display(edit_state.anchor),
    );
    core.draw_list.push_scissor(rect);
    if focused {
        let mut selection_color = color_to_f32(theme.info);
        selection_color[3] = 0.35;
        for selection in geometry.selection {
            if let Some(quad) = text_edit_quad(
                [
                    text_x + selection[0],
                    text_y + selection[1],
                    selection[2],
                    selection[3],
                ],
                selection_color,
                0.01,
                rect,
            ) {
                core.draw_list.push_quad(quad);
            }
        }
    }

    core.draw_list.push_text(TextCall {
        buffer_id,
        x: text_x,
        y: text_y,
        clip: rect,
        color: color_to_f32(text_color),
        z: 0.0,
    });

    if focused && cursor_visible {
        if let Some(caret) = geometry.caret {
            if let Some(quad) = text_edit_quad(
                [text_x + caret[0], text_y + caret[1], caret[2], caret[3]],
                color_to_f32(theme.primary),
                Z_TEXT_FOREGROUND,
                rect,
            ) {
                core.draw_list.push_quad(quad);
            }
        }
    }
    core.draw_list.pop_scissor();

    TextInputResponse {
        changed,
        submitted,
        copy_text,
        request_paste,
    }
}

/// Ensures the widget's stable buffer id reflects `value` as shaped right
/// now, so a same-frame `Left`/`Right` key event always drives cosmic-text's
/// `Buffer::cursor_motion` off the paragraph this keystroke actually acted
/// on — not a shape left over from the previous frame, or from an earlier
/// edit (insert/backspace/delete) processed earlier in this same frame's
/// key-event loop. `set_text` is idempotent and already runs unconditionally
/// once per frame for rendering, so re-running it here on demand is the
/// smallest lifecycle-safe way to guarantee a fresh shape at the point of
/// use, rather than restructuring the whole frame around a "shape first, then
/// handle keys" ordering.
fn reshape_for_motion(
    core: &mut AkarCore,
    id_u64: u64,
    value: &str,
    masked: bool,
    metrics: glyphon::Metrics,
    max_text_width: f32,
) -> ShapedForMotion {
    let display = if masked && !value.is_empty() {
        "*".repeat(value.chars().count())
    } else {
        value.to_string()
    };
    let buffer_id = core.text_pipeline.set_text(
        Some(id_u64),
        &display,
        metrics,
        Some(max_text_width),
        None,
        None,
    );
    ShapedForMotion {
        buffer_id,
        offsets: TextOffsetMap::new(value, masked),
    }
}

struct ShapedForMotion {
    buffer_id: u64,
    offsets: TextOffsetMap,
}

enum TextOffsetMap {
    Identity,
    Masked { original_boundaries: Vec<usize> },
}

impl TextOffsetMap {
    fn new(value: &str, masked: bool) -> Self {
        if !masked {
            return Self::Identity;
        }
        Self::Masked {
            original_boundaries: value
                .char_indices()
                .map(|(offset, _)| offset)
                .chain(std::iter::once(value.len()))
                .collect(),
        }
    }

    fn to_display(&self, original_offset: usize) -> usize {
        match self {
            Self::Identity => original_offset,
            Self::Masked {
                original_boundaries,
            } => original_boundaries
                .partition_point(|boundary| *boundary <= original_offset)
                .saturating_sub(1),
        }
    }

    fn to_original(&self, display_offset: usize) -> usize {
        match self {
            Self::Identity => display_offset,
            Self::Masked {
                original_boundaries,
            } => original_boundaries
                .get(display_offset)
                .copied()
                .unwrap_or_else(|| original_boundaries.last().copied().unwrap_or(0)),
        }
    }
}

fn collapse_selection_for_motion(
    pipeline: &akar_core::TextPipeline,
    shaped: &ShapedForMotion,
    state: &mut TextEditState,
    direction: CaretMotion,
) {
    let mut display_state = TextEditState {
        cursor: shaped.offsets.to_display(state.cursor),
        anchor: shaped.offsets.to_display(state.anchor),
    };
    collapse_selection_visually(pipeline, shaped.buffer_id, &mut display_state, direction);
    state.cursor = shaped.offsets.to_original(display_state.cursor);
    state.anchor = state.cursor;
}

fn text_edit_quad(rect: [f32; 4], fill: [f32; 4], z: f32, clip: [f32; 4]) -> Option<QuadCall> {
    let x = rect[0].max(clip[0]);
    let y = rect[1].max(clip[1]);
    let width = (rect[0] + rect[2]).min(clip[0] + clip[2]) - x;
    let height = (rect[1] + rect[3]).min(clip[1] + clip[3]) - y;
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    Some(QuadCall {
        rect: [x, y, width, height],
        fill,
        border_color: [0.0; 4],
        corner_radii: [0.0; 4],
        border_width: 0.0,
        z,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    #[test]
    fn zero_area_returns_default() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());

        let mut core = AkarCore::mock();
        let mut value = String::new();
        let mut edit_state = TextEditState::default();

        let result = text_input(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            "Placeholder",
            true,
            &AKAR_THEME_DARK,
        );

        assert!(!result.changed);
        assert!(!result.submitted);
    }

    fn focused_field(value: &str) -> (AkarCore, Layout, NodeId, String, TextEditState) {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(300.0_f32),
                height: akar_layout::length(40.0_f32),
            },
            ..Default::default()
        });
        layout.compute(node_id, (Some(400.0), Some(400.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });

        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let id_u64 = layout.widget_id(node_id);
        core.input.focused_id = Some(id_u64);

        let edit_state = TextEditState {
            cursor: value.len(),
            anchor: value.len(),
        };
        (core, layout, node_id, value.to_string(), edit_state)
    }

    fn press_arrow(
        core: &mut AkarCore,
        layout: &Layout,
        node_id: NodeId,
        value: &mut String,
        edit_state: &mut TextEditState,
        key: Key,
    ) {
        press_arrow_masked(core, layout, node_id, value, edit_state, key, false);
    }

    fn press_arrow_masked(
        core: &mut AkarCore,
        layout: &Layout,
        node_id: NodeId,
        value: &mut String,
        edit_state: &mut TextEditState,
        key: Key,
        masked: bool,
    ) {
        core.input.begin_frame();
        core.input.focused_id = Some(layout.widget_id(node_id));
        core.input.push_key(key);
        text_input_masked(
            core,
            layout,
            node_id,
            value,
            edit_state,
            "",
            true,
            &AKAR_THEME_DARK,
            masked,
        );
    }

    #[test]
    fn masked_multibyte_ltr_arrows_preserve_original_utf8_boundaries() {
        let (mut core, layout, node_id, mut value, mut edit_state) = focused_field("éa");
        edit_state.cursor = 2;
        edit_state.anchor = 2;

        press_arrow_masked(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            Key::Left,
            true,
        );
        assert_eq!(edit_state.cursor, 0);
        assert!(value.is_char_boundary(edit_state.cursor));

        press_arrow_masked(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            Key::Right,
            true,
        );
        assert_eq!(edit_state.cursor, 2);
        assert!(value.is_char_boundary(edit_state.cursor));
    }

    #[test]
    fn masked_multibyte_rtl_arrows_and_selection_preserve_original_utf8_boundaries() {
        let text = "שלום";
        let (mut core, layout, node_id, mut value, mut edit_state) = focused_field(text);
        let first_character_end = text.chars().next().unwrap().len_utf8();
        edit_state.cursor = first_character_end;
        edit_state.anchor = first_character_end;

        press_arrow_masked(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            Key::Right,
            true,
        );
        assert_eq!(edit_state.cursor, first_character_end * 2);
        assert!(value.is_char_boundary(edit_state.cursor));

        edit_state.anchor = 0;
        press_arrow_masked(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            Key::Left,
            true,
        );
        assert_eq!(edit_state, TextEditState::default());
    }

    #[test]
    fn unmasked_multibyte_arrow_motion_keeps_shaped_rtl_behavior() {
        let text = "שלום";
        let (mut core, layout, node_id, mut value, mut edit_state) = focused_field(text);
        edit_state.cursor = 0;
        edit_state.anchor = 0;

        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            Key::Left,
        );
        assert_ne!(edit_state.cursor, 0);
        assert!(value.is_char_boundary(edit_state.cursor));
    }

    /// ASCII alone cannot distinguish "moved by app layout direction" from
    /// "moved by the shaped paragraph's own direction," since both would
    /// agree on plain LTR text. This drives the arrow keys through the real
    /// `text_input` key-event loop (not the pipeline adapter directly) for
    /// real Arabic text, and checks that physical `Right` is a no-op at a
    /// pure-RTL paragraph's logical start (its visually-rightmost, leading
    /// edge), while `Left` advances.
    #[test]
    fn arrow_keys_follow_shaped_paragraph_direction_for_arabic_value() {
        let (mut core, layout, node_id, mut value, mut edit_state) = focused_field("مرحبا");
        edit_state.cursor = 0;
        edit_state.anchor = 0;

        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            Key::Right,
        );
        assert_eq!(
            edit_state.cursor, 0,
            "Right at the start of an RTL value must not move logically backward"
        );

        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            Key::Left,
        );
        assert_ne!(
            edit_state.cursor, 0,
            "Left at the start of an RTL value must advance logically"
        );
    }

    /// Same wiring, but for Hebrew — a second RTL script, and an LTR value
    /// on the same code path so the two are contrasted rather than assumed.
    #[test]
    fn arrow_keys_follow_shaped_paragraph_direction_for_hebrew_and_ltr_values() {
        let (mut core, layout, node_id, mut value, mut edit_state) = focused_field("שלום");
        edit_state.cursor = 0;
        edit_state.anchor = 0;
        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            Key::Right,
        );
        assert_eq!(edit_state.cursor, 0);

        let (mut core, layout, node_id, mut value, mut edit_state) = focused_field("value");
        edit_state.cursor = 0;
        edit_state.anchor = 0;
        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            Key::Right,
        );
        assert_eq!(
            edit_state.cursor, 1,
            "Right on an LTR value must advance logically"
        );
    }

    /// Pressing an arrow with an active selection must collapse to the
    /// visually-left/right endpoint, not the smaller/larger byte offset —
    /// verified end to end through the component, on top of the
    /// `text_edit::collapse_selection_visually` unit coverage.
    #[test]
    fn right_arrow_collapses_rtl_selection_to_visual_right_endpoint() {
        let text = "שלום";
        let (mut core, layout, node_id, mut value, mut edit_state) = focused_field(text);
        edit_state.anchor = 0;
        edit_state.cursor = text.len();

        press_arrow(
            &mut core,
            &layout,
            node_id,
            &mut value,
            &mut edit_state,
            Key::Right,
        );

        assert_eq!(
            edit_state,
            TextEditState {
                cursor: 0,
                anchor: 0
            },
            "visual-right of an RTL selection is its logical start, byte offset 0"
        );
    }

    #[test]
    fn text_edit_geometry_is_clipped_to_field() {
        let quad = text_edit_quad(
            [5.0, -5.0, 20.0, 20.0],
            [1.0; 4],
            Z_TEXT_FOREGROUND,
            [10.0, 0.0, 10.0, 10.0],
        )
        .expect("partially visible");
        assert_eq!(quad.rect, [10.0, 0.0, 10.0, 10.0]);
        assert!(text_edit_quad(
            [30.0, 30.0, 2.0, 10.0],
            [1.0; 4],
            Z_TEXT_FOREGROUND,
            [10.0, 0.0, 10.0, 10.0]
        )
        .is_none());
    }
}
