use akar_core::{AkarCore, Key, QuadCall, TextCall, Z_BASE, Z_TEXT_FOREGROUND};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;

pub const DATA_GRID_OVERSCAN: usize = 2;
const HEADER_DOMAIN_OFFSET: u64 = 1u64 << 62;

fn normalize_dimension(v: f32, fallback: f32) -> f32 {
    if v.is_finite() && v > 0.0 {
        v
    } else {
        fallback
    }
}

fn normalize_width(v: f32) -> f32 {
    if v.is_finite() && v > 0.0 {
        v
    } else {
        0.0
    }
}

pub fn total_content_width(columns: &[DataGridColumn]) -> f32 {
    columns.iter().map(|c| normalize_width(c.width)).sum()
}

pub fn total_content_height(row_count: usize, row_height: f32) -> f32 {
    row_count as f32 * row_height
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataGridAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataGridSortDirection {
    None,
    Ascending,
    Descending,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataGridColumn {
    pub key: u64,
    pub width: f32,
    pub align: DataGridAlign,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataGridStyle {
    pub header_bg: u32,
    pub header_text: u32,
    pub row_bg: u32,
    pub row_bg_alt: u32,
    pub row_text: u32,
    pub selected_row_bg: u32,
    pub selected_row_text: u32,
    pub active_cell_bg: u32,
    pub active_cell_text: u32,
    pub hover_bg: u32,
    pub grid_line_color: u32,
    pub grid_line_width: f32,
    pub header_height: f32,
    pub row_height: f32,
    pub cell_padding_x: f32,
    pub cell_padding_y: f32,
    pub font_size: f32,
}

impl DataGridStyle {
    pub fn from_theme(theme: &crate::theme::AkarTheme) -> Self {
        Self {
            header_bg: theme.secondary,
            header_text: theme.secondary_content,
            row_bg: theme.base_100,
            row_bg_alt: theme.base_200,
            row_text: theme.base_content,
            selected_row_bg: theme.primary,
            selected_row_text: theme.primary_content,
            active_cell_bg: theme.accent,
            active_cell_text: theme.accent_content,
            hover_bg: theme.base_300,
            grid_line_color: theme.base_300,
            grid_line_width: theme.border_width,
            header_height: 36.0,
            row_height: 32.0,
            cell_padding_x: theme.padding_x,
            cell_padding_y: theme.padding_y,
            font_size: theme.font_size_base,
        }
    }
}

fn rect_intersect(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let x = a[0].max(b[0]);
    let y = a[1].max(b[1]);
    let right = (a[0] + a[2]).min(b[0] + b[2]);
    let bottom = (a[1] + a[3]).min(b[1] + b[3]);
    let w = (right - x).max(0.0);
    let h = (bottom - y).max(0.0);
    [x, y, w, h]
}

fn cell_padding_x(col_width: f32, font_size: f32) -> f32 {
    let p = font_size * 0.75;
    if p * 2.0 >= col_width {
        0.0
    } else {
        p
    }
}

/// Caller-owned scroll and active-cell state. akar mutates this immediately
/// before drawing each frame (scroll clamping, active-cell tracking).
/// The caller persists this across frames and may reset or synchronize it.
pub struct DataGridState {
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub active_row_key: u64,
    pub active_column_key: u64,
    pub has_active_cell: bool,
}

impl DataGridState {
    pub fn new() -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            active_row_key: 0,
            active_column_key: 0,
            has_active_cell: false,
        }
    }
}

impl Default for DataGridState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataGridCellRef {
    pub row_key: u64,
    pub column_key: u64,
    pub row_index: usize,
    pub column_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataGridResponse {
    pub viewport_rect: [f32; 4],
    pub header_rect: [f32; 4],
    pub body_rect: [f32; 4],
    pub visible_rows: std::ops::Range<usize>,
    pub visible_columns: std::ops::Range<usize>,
    pub activated: Option<DataGridCellRef>,
    pub header_clicked: Option<u64>,
    pub row_height: f32,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub total_content_width: f32,
    pub total_content_height: f32,
    pub has_active_cell: bool,
    pub active_row_key: u64,
    pub column_offsets: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataGridHeaderResponse {
    pub rect: [f32; 4],
    pub column_key: u64,
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataGridCellResponse {
    pub rect: [f32; 4],
    pub row_key: u64,
    pub column_key: u64,
    pub row_index: usize,
    pub column_index: usize,
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DataGridKeyboardResponse {
    pub activated: bool,
    pub cell_changed: bool,
}

/// Cumulative column offsets for virtualization. Computed once per frame
/// from the column descriptors. Capped at column count to bound iteration.
struct ColumnLayout {
    offsets: Vec<f32>,
    total_width: f32,
}

fn compute_column_layout(columns: &[DataGridColumn]) -> ColumnLayout {
    let mut offsets = Vec::with_capacity(columns.len() + 1);
    offsets.push(0.0);
    for col in columns {
        let w = normalize_width(col.width);
        let prev = *offsets.last().unwrap();
        offsets.push(prev + w);
    }
    let total_width = *offsets.last().unwrap();
    ColumnLayout {
        offsets,
        total_width,
    }
}

fn visible_column_range(
    offsets: &[f32],
    scroll_x: f32,
    viewport_width: f32,
    total_columns: usize,
) -> std::ops::Range<usize> {
    if total_columns == 0 || offsets.len() <= 1 {
        return 0..0;
    }
    let mut start = 0;
    for i in 0..total_columns {
        if offsets[i + 1] > scroll_x {
            start = i;
            break;
        }
        if i == total_columns - 1 {
            start = total_columns;
        }
    }
    let end_x = scroll_x + viewport_width;
    let mut end = start;
    for (i, offset) in offsets
        .iter()
        .enumerate()
        .skip(start)
        .take(total_columns - start)
    {
        end = i + 1;
        if *offset >= end_x {
            break;
        }
    }
    let overscan_start = start.saturating_sub(DATA_GRID_OVERSCAN);
    let overscan_end = (end + DATA_GRID_OVERSCAN).min(total_columns);
    overscan_start..overscan_end
}

fn visible_row_range(
    row_count: usize,
    row_height: f32,
    scroll_y: f32,
    body_height: f32,
) -> std::ops::Range<usize> {
    if row_count == 0 || row_height <= 0.0 || body_height <= 0.0 {
        return 0..0;
    }
    let base = akar_core::list_clip(row_count, row_height, scroll_y, body_height);
    let start = base.start.saturating_sub(DATA_GRID_OVERSCAN);
    let end = (base.end + DATA_GRID_OVERSCAN).min(row_count);
    start..end
}

fn find_active_row_index(row_keys: &[u64], active_row_key: u64) -> Option<usize> {
    row_keys.iter().position(|&k| k == active_row_key)
}

/// Computes new scroll_x and scroll_y to bring the given cell rect fully
/// into the viewport. The cell rect is in content coordinates (not screen
/// coordinates). Returns (new_scroll_x, new_scroll_y).
///
/// If the cell is already fully visible, returns the current scroll values unchanged.
/// If the cell is larger than the viewport, scrolls to show its top-left corner.
#[allow(clippy::too_many_arguments)]
pub fn scroll_to_reveal(
    cell_x: f32,
    cell_y: f32,
    cell_w: f32,
    cell_h: f32,
    viewport_w: f32,
    viewport_h: f32,
    scroll_x: f32,
    scroll_y: f32,
) -> (f32, f32) {
    let mut new_sx = scroll_x;
    let mut new_sy = scroll_y;

    let cell_right = cell_x + cell_w;
    let cell_bottom = cell_y + cell_h;
    let view_right = scroll_x + viewport_w;
    let view_bottom = scroll_y + viewport_h;

    if cell_x < scroll_x {
        new_sx = cell_x;
    } else if cell_right > view_right {
        new_sx = cell_right - viewport_w;
    }

    if cell_y < scroll_y {
        new_sy = cell_y;
    } else if cell_bottom > view_bottom {
        new_sy = cell_bottom - viewport_h;
    }

    (new_sx.max(0.0), new_sy.max(0.0))
}

/// Begins a virtualized data-grid scope.
///
/// Lifecycle:
///   data_grid_begin
///   -> data_grid_header_begin
///      -> [data_grid_header_cell]*
///   -> data_grid_header_end
///   -> data_grid_body_begin
///      -> [data_grid_cell]*
///   -> data_grid_body_end
///   data_grid_end
///
/// `row_keys` must have length equal to `row_count`. Each key is a stable
/// caller-owned identifier for the row at that index in the current sort/filter
/// order. When rows are sorted, filtered, or removed between frames, the caller
/// supplies the new key sequence; akar matches the active cell by key, not by
/// screen position. If the active row key is absent from the new row_keys,
/// has_active_cell is cleared and no row is highlighted.
///
/// Columns with non-finite or negative widths are normalized to 0.0.
/// row_height and header_height are normalized to documented defaults if
/// non-finite or non-positive.
///
/// Zero row_count, zero columns, zero-area viewports, and columns wider than
/// the viewport are valid and produce empty visible ranges rather than panics.
///
/// akar mutates state.scroll_x and state.scroll_y to clamped values before
/// drawing. The caller should persist DataGridState across frames.
#[allow(clippy::too_many_arguments)]
pub fn data_grid_begin(
    core: &mut AkarCore,
    layout: &Layout,
    node: NodeId,
    state: &mut DataGridState,
    row_count: usize,
    row_keys: &[u64],
    row_height: f32,
    header_height: f32,
    columns: &[DataGridColumn],
    style: &DataGridStyle,
) -> DataGridResponse {
    let rect = layout.rect(node);
    let [x, y, w, h] = rect;

    let row_height = normalize_dimension(row_height, style.row_height);
    let header_height = normalize_dimension(header_height, style.header_height);

    let col_layout = compute_column_layout(columns);

    let content_height = row_count as f32 * row_height;
    let content_width = col_layout.total_width;
    let body_height = (h - header_height).max(0.0);

    if core.input.is_hovering(rect) {
        state.scroll_x -= core.input.scroll_delta.x;
        state.scroll_y -= core.input.scroll_delta.y;
    }

    let max_scroll_x = (content_width - w).max(0.0);
    let max_scroll_y = (content_height - body_height).max(0.0);
    state.scroll_x = state.scroll_x.clamp(0.0, max_scroll_x);
    state.scroll_y = state.scroll_y.clamp(0.0, max_scroll_y);

    if state.has_active_cell && find_active_row_index(row_keys, state.active_row_key).is_none() {
        state.has_active_cell = false;
    }

    let header_rect = [x, y, w, header_height];
    let body_rect = [x, y + header_height, w, body_height];

    let visible_rows = visible_row_range(row_count, row_height, state.scroll_y, body_height);
    let visible_columns =
        visible_column_range(&col_layout.offsets, state.scroll_x, w, columns.len());

    if w > 0.0 && h > 0.0 {
        core.draw_list.push_quad(QuadCall {
            rect,
            fill: color_to_f32(style.row_bg),
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: Z_BASE,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });
    }

    core.draw_list.push_scissor(rect);

    DataGridResponse {
        viewport_rect: rect,
        header_rect,
        body_rect,
        visible_rows,
        visible_columns,
        activated: None,
        header_clicked: None,
        row_height,
        scroll_x: state.scroll_x,
        scroll_y: state.scroll_y,
        total_content_width: col_layout.total_width,
        total_content_height: content_height,
        has_active_cell: state.has_active_cell,
        active_row_key: state.active_row_key,
        column_offsets: col_layout.offsets,
    }
}

/// Pushes a scissor for the sticky header region. The header scrolls
/// horizontally with the body but stays pinned vertically. Draws the
/// sticky header background quad before pushing the scissor.
pub fn data_grid_header_begin(
    core: &mut AkarCore,
    response: &DataGridResponse,
    style: &DataGridStyle,
) {
    let [hx, hy, hw, hh] = response.header_rect;
    if hw > 0.0 && hh > 0.0 {
        core.draw_list.push_quad(QuadCall {
            rect: [hx, hy, hw, hh],
            fill: color_to_f32(style.header_bg),
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: Z_BASE,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });
    }
    core.draw_list.push_scissor(response.header_rect);
}

/// Returns the rect for a single header cell and detects click on it.
///
/// The caller is responsible for drawing the header cell content after
/// receiving this response. The `sort` parameter is the caller-owned current
/// sort state for this column; akar does not mutate it.
///
/// The header scissor (pushed by data_grid_header_begin) clips cells outside
/// the viewport. Hit-testing uses the unclipped arithmetic rect; the scissor
/// prevents drawing but a partially visible header cell is still clickable
/// within its visible portion.
#[allow(clippy::too_many_arguments)]
pub fn data_grid_header_cell(
    core: &mut AkarCore,
    layout: &Layout,
    response: &DataGridResponse,
    node: NodeId,
    column_index: usize,
    columns: &[DataGridColumn],
    style: &DataGridStyle,
    label: &str,
    sort: DataGridSortDirection,
) -> DataGridHeaderResponse {
    if column_index >= columns.len() {
        return DataGridHeaderResponse {
            rect: [0.0; 4],
            column_key: 0,
            hovered: false,
            pressed: false,
            clicked: false,
        };
    }

    let [vx, vy, _, _] = response.viewport_rect;
    let [_, _, _, hh] = response.header_rect;

    let col = &columns[column_index];
    let col_x = vx + response.column_offsets[column_index] - response.scroll_x;
    let col_w = normalize_width(col.width);
    let rect = [col_x, vy, col_w, hh];

    let hit_rect = core
        .draw_list
        .active_scissor()
        .map_or(rect, |s| rect_intersect(rect, s));
    let hovered = core.input.is_hovering(hit_rect);
    let pressed = hovered && core.input.mouse_buttons[0];
    let clicked = core.input.is_clicked(hit_rect);

    let cell_clip = [col_x, vy, col_w, hh];
    let font_size = style.font_size;
    let px = cell_padding_x(col_w, font_size);
    let py = (hh - font_size) * 0.5;

    let base_id = layout.widget_id_keyed(node, col.key);
    let hdr_id = base_id | HEADER_DOMAIN_OFFSET;

    if !label.is_empty() {
        let buffer_id = core.text_pipeline.set_text(
            Some(hdr_id),
            label,
            glyphon::Metrics::new(font_size, font_size * 1.2),
            Some(col_w - px * 2.0),
            None,
            None,
        );
        core.draw_list.push_text(TextCall {
            buffer_id,
            x: col_x + px,
            y: vy + py,
            clip: cell_clip,
            color: color_to_f32(style.header_text),
            z: Z_TEXT_FOREGROUND,
        });
    }

    let indicator = match sort {
        DataGridSortDirection::Ascending => "\u{25B2}",
        DataGridSortDirection::Descending => "\u{25BC}",
        DataGridSortDirection::None => "",
    };
    if !indicator.is_empty() {
        let sort_id = hdr_id ^ 0xA5A5_A5A5_A5A5_A5A5;
        let ind_size = font_size * 0.6;
        let ind_buf = core.text_pipeline.set_text(
            Some(sort_id),
            indicator,
            glyphon::Metrics::new(ind_size, ind_size * 1.2),
            None,
            None,
            None,
        );
        core.draw_list.push_text(TextCall {
            buffer_id: ind_buf,
            x: col_x + col_w - px - ind_size,
            y: vy + py,
            clip: cell_clip,
            color: color_to_f32(style.header_text),
            z: Z_TEXT_FOREGROUND,
        });
    }

    DataGridHeaderResponse {
        rect,
        column_key: col.key,
        hovered,
        pressed,
        clicked,
    }
}

/// Pops the header scissor.
pub fn data_grid_header_end(core: &mut AkarCore) {
    core.draw_list.pop_scissor();
}

/// Pushes a scissor for the body region. Body cells are clipped to this rect
/// and cannot paint over the sticky header. Draws alternating row backgrounds,
/// grid lines, selected-row highlights, and the active-cell highlight.
pub fn data_grid_body_begin(
    core: &mut AkarCore,
    response: &DataGridResponse,
    row_keys: &[u64],
    style: &DataGridStyle,
    selected_rows: &[u64],
) {
    let [bx, by, bw, bh] = response.body_rect;
    core.draw_list.push_scissor(response.body_rect);

    if bw <= 0.0 || bh <= 0.0 {
        return;
    }

    let glw = style.grid_line_width;
    let grid_line = if glw > 0.0 {
        color_to_f32(style.grid_line_color)
    } else {
        [0.0; 4]
    };

    for row_i in response.visible_rows.clone() {
        let row_y = by + (row_i as f32) * response.row_height - response.scroll_y;
        let row_bg = if row_i % 2 == 0 {
            style.row_bg
        } else {
            style.row_bg_alt
        };
        core.draw_list.push_quad(QuadCall {
            rect: [bx, row_y, bw, response.row_height],
            fill: color_to_f32(row_bg),
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: Z_BASE,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let key = if row_i < row_keys.len() {
            row_keys[row_i]
        } else {
            0
        };
        if selected_rows.contains(&key) {
            core.draw_list.push_quad(QuadCall {
                rect: [bx, row_y, bw, response.row_height],
                fill: color_to_f32(style.selected_row_bg),
                border_color: [0.0; 4],
                corner_radii: [0.0; 4],
                border_width: 0.0,
                z: Z_BASE + 0.01,
                shadow_blur: 0.0,
                shadow_spread: 0.0,
                shadow_color: [0.0; 4],
                shadow_offset: [0.0; 2],
                _pad: [0.0; 2],
            });
        }
    }

    if response.visible_rows.start < response.visible_rows.end {
        let last_vis = response.visible_rows.end as f32;
        let content_top = by;
        let content_bottom = by + last_vis * response.row_height - response.scroll_y;
        let glh = glw.max(1.0);

        for row_i in response.visible_rows.clone() {
            if row_i == 0 {
                continue;
            }
            let line_y = by + (row_i as f32) * response.row_height - response.scroll_y - glh * 0.5;
            core.draw_list.push_quad(QuadCall {
                rect: [bx, line_y, bw, glh],
                fill: grid_line,
                border_color: [0.0; 4],
                corner_radii: [0.0; 4],
                border_width: 0.0,
                z: Z_BASE + 0.02,
                shadow_blur: 0.0,
                shadow_spread: 0.0,
                shadow_color: [0.0; 4],
                shadow_offset: [0.0; 2],
                _pad: [0.0; 2],
            });
        }

        let col_offsets = &response.column_offsets;
        let vx = response.viewport_rect[0];
        let scroll_x = response.scroll_x;
        for offset in col_offsets
            .iter()
            .skip(1)
            .take(col_offsets.len().saturating_sub(2))
        {
            let line_x = vx + offset - scroll_x - glh * 0.5;
            if line_x + glh < bx || line_x > bx + bw {
                continue;
            }
            core.draw_list.push_quad(QuadCall {
                rect: [line_x, content_top, glh, content_bottom - content_top],
                fill: grid_line,
                border_color: [0.0; 4],
                corner_radii: [0.0; 4],
                border_width: 0.0,
                z: Z_BASE + 0.02,
                shadow_blur: 0.0,
                shadow_spread: 0.0,
                shadow_color: [0.0; 4],
                shadow_offset: [0.0; 2],
                _pad: [0.0; 2],
            });
        }

        if (style.active_cell_text != 0 || style.active_cell_bg != 0) && response.has_active_cell {
            if let Some(active_idx) = find_active_row_index(row_keys, response.active_row_key) {
                if response.visible_rows.contains(&active_idx) {
                    let row_y = by + (active_idx as f32) * response.row_height - response.scroll_y;
                    core.draw_list.push_quad(QuadCall {
                        rect: [bx, row_y, bw, response.row_height],
                        fill: color_to_f32(style.active_cell_bg),
                        border_color: [0.0; 4],
                        corner_radii: [0.0; 4],
                        border_width: 0.0,
                        z: Z_BASE + 0.015,
                        shadow_blur: 0.0,
                        shadow_spread: 0.0,
                        shadow_color: [0.0; 4],
                        shadow_offset: [0.0; 2],
                        _pad: [0.0; 2],
                    });
                }
            }
        }
    }
}

/// Returns the rect for a single body cell and detects click/activation.
///
/// When a cell is clicked, the response reports it via the return value.
/// The caller also receives the click through the returned `DataGridCellResponse`.
/// The caller should then update `state.active_row_key`, `state.active_column_key`,
/// and `state.has_active_cell` based on its own policy. akar does not mutate
/// active-cell state in this function; that happens in data_grid_begin on the
/// next frame, or the caller can set it immediately.
///
/// `selected_row` is caller-owned selection state for this row; akar applies
/// the selected-row style but does not own the selection set.
#[allow(clippy::too_many_arguments)]
pub fn data_grid_cell(
    core: &mut AkarCore,
    layout: &Layout,
    response: &DataGridResponse,
    node: NodeId,
    row_index: usize,
    row_key: u64,
    column_index: usize,
    columns: &[DataGridColumn],
    style: &DataGridStyle,
    text: &str,
    selected_row: bool,
) -> DataGridCellResponse {
    if column_index >= columns.len() {
        return DataGridCellResponse {
            rect: [0.0; 4],
            row_key: 0,
            column_key: 0,
            row_index: 0,
            column_index: 0,
            hovered: false,
            pressed: false,
            clicked: false,
        };
    }

    let [vx, _, _, _] = response.viewport_rect;
    let [_, body_y, _, _] = response.body_rect;
    let row_height = response.row_height;
    let cell_y = body_y + (row_index as f32) * row_height - response.scroll_y;
    let col = &columns[column_index];
    let col_x = vx + response.column_offsets[column_index] - response.scroll_x;
    let col_w = normalize_width(col.width);
    let rect = [col_x, cell_y, col_w, row_height];

    let hit_rect = core
        .draw_list
        .active_scissor()
        .map_or(rect, |s| rect_intersect(rect, s));
    let hovered = core.input.is_hovering(hit_rect);
    let pressed = hovered && core.input.mouse_buttons[0];
    let clicked = core.input.is_clicked(hit_rect);

    if clicked {
        core.input.focused_id = Some(layout.widget_id(node));
    }

    if hovered {
        core.draw_list.push_quad(QuadCall {
            rect,
            fill: color_to_f32(style.hover_bg),
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: Z_BASE + 0.005,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });
    }

    if !text.is_empty() {
        let font_size = style.font_size;
        let px = cell_padding_x(col_w, font_size);
        let py = (row_height - font_size) * 0.5;
        let row_base = layout.widget_id_keyed(node, row_key);
        let cell_id = (row_base ^ col.key) & !HEADER_DOMAIN_OFFSET;

        let text_color = if selected_row {
            style.selected_row_text
        } else {
            style.row_text
        };

        let text_w = (col_w - px * 2.0).max(0.0);
        let text_x = match col.align {
            DataGridAlign::Left => col_x + px,
            DataGridAlign::Center => col_x + (col_w - text_w) * 0.5,
            DataGridAlign::Right => col_x + col_w - px - text_w,
        };

        let buffer_id = core.text_pipeline.set_text_transient(
            cell_id,
            text,
            glyphon::Metrics::new(font_size, font_size * 1.2),
            Some(text_w),
            None,
            None,
        );
        core.draw_list.push_text(TextCall {
            buffer_id,
            x: text_x,
            y: cell_y + py,
            clip: rect,
            color: color_to_f32(text_color),
            z: Z_TEXT_FOREGROUND,
        });
    }

    let _ = selected_row;

    DataGridCellResponse {
        rect,
        row_key,
        column_key: col.key,
        row_index,
        column_index,
        hovered,
        pressed,
        clicked,
    }
}

/// Pops the body scissor.
pub fn data_grid_body_end(core: &mut AkarCore) {
    core.draw_list.pop_scissor();
}

/// Pops the outer viewport scissor pushed by data_grid_begin.
/// Must be called after data_grid_body_end (or data_grid_header_end if
/// no body is rendered).
pub fn data_grid_end(core: &mut AkarCore) {
    core.draw_list.pop_scissor();
}

/// Processes keyboard input for grid navigation while the grid owns focus.
///
/// Call before `data_grid_begin`. Updates `state.active_row_key`,
/// `state.active_column_key`, `state.has_active_cell`, `state.scroll_x`,
/// and `state.scroll_y`. The caller should then call `data_grid_begin`
/// which will clamp scroll and validate the active cell against current data.
///
/// Navigation keys:
/// - Up/Down/Left/Right: move active cell one step
/// - Home/End: first/last column; with Ctrl: first/last row
/// - PageUp/PageDown: move by visible page of rows
/// - Tab/Shift+Tab: next/previous column, wrapping between rows
/// - Enter: activation response for current cell
/// - Escape: release grid focus
#[allow(clippy::too_many_arguments)]
pub fn data_grid_handle_keyboard(
    core: &mut AkarCore,
    layout: &Layout,
    node: NodeId,
    state: &mut DataGridState,
    row_count: usize,
    row_keys: &[u64],
    columns: &[DataGridColumn],
    style: &DataGridStyle,
) -> DataGridKeyboardResponse {
    let grid_focus_id = layout.widget_id_keyed(node, 0);
    if core.input.focused_id != Some(grid_focus_id) {
        return DataGridKeyboardResponse::default();
    }

    if columns.is_empty() || row_count == 0 || row_keys.is_empty() {
        for ev in &core.input.key_events {
            if ev.key == Key::Escape {
                core.input.focused_id = None;
                break;
            }
        }
        return DataGridKeyboardResponse::default();
    }

    let col_layout = compute_column_layout(columns);
    let rect = layout.rect(node);
    let row_height = normalize_dimension(style.row_height, 32.0);
    let header_height = normalize_dimension(style.header_height, 36.0);
    let viewport_w = rect[2];
    let body_height = (rect[3] - header_height).max(0.0);

    let mut cur_row = if state.has_active_cell {
        find_active_row_index(row_keys, state.active_row_key)
    } else {
        None
    };
    let mut cur_col = if state.has_active_cell {
        columns
            .iter()
            .position(|c| c.key == state.active_column_key)
    } else {
        None
    };

    if state.has_active_cell {
        if cur_row.is_none() && row_count > 0 {
            cur_row = Some(row_count - 1);
        }
        if cur_col.is_none() && !columns.is_empty() {
            cur_col = Some(columns.len() - 1);
        }
    }

    let mut cell_changed = false;
    let mut activated = false;

    let has_nav_key = core.input.key_events.iter().any(|ev| {
        matches!(
            ev.key,
            Key::Up
                | Key::Down
                | Key::Left
                | Key::Right
                | Key::Home
                | Key::End
                | Key::Tab
                | Key::PageUp
                | Key::PageDown
        )
    });

    if has_nav_key && cur_row.is_none() && cur_col.is_none() {
        cur_row = Some(0);
        cur_col = Some(0);
        cell_changed = true;
    }

    for ev in &core.input.key_events {
        match ev.key {
            Key::Up => {
                if let Some(r) = cur_row {
                    if r > 0 {
                        cur_row = Some(r - 1);
                        cell_changed = true;
                    }
                }
            }
            Key::Down => {
                if let Some(r) = cur_row {
                    if r + 1 < row_count {
                        cur_row = Some(r + 1);
                        cell_changed = true;
                    }
                }
            }
            Key::Left => {
                if let Some(c) = cur_col {
                    if c > 0 {
                        cur_col = Some(c - 1);
                        cell_changed = true;
                    }
                }
            }
            Key::Right => {
                if let Some(c) = cur_col {
                    if c + 1 < columns.len() {
                        cur_col = Some(c + 1);
                        cell_changed = true;
                    }
                }
            }
            Key::Home => {
                if ev.modifiers.control {
                    if let Some(r) = cur_row {
                        if r != 0 {
                            cur_row = Some(0);
                            cell_changed = true;
                        }
                    }
                } else if let Some(c) = cur_col {
                    if c != 0 {
                        cur_col = Some(0);
                        cell_changed = true;
                    }
                }
            }
            Key::End => {
                if ev.modifiers.control {
                    if let Some(r) = cur_row {
                        let last = row_count - 1;
                        if r != last {
                            cur_row = Some(last);
                            cell_changed = true;
                        }
                    }
                } else if let Some(c) = cur_col {
                    let last = columns.len() - 1;
                    if c != last {
                        cur_col = Some(last);
                        cell_changed = true;
                    }
                }
            }
            Key::Tab => {
                if let (Some(r), Some(c)) = (cur_row, cur_col) {
                    if ev.modifiers.shift {
                        if c > 0 {
                            cur_col = Some(c - 1);
                        } else if r > 0 {
                            cur_row = Some(r - 1);
                            cur_col = Some(columns.len() - 1);
                        }
                    } else if c + 1 < columns.len() {
                        cur_col = Some(c + 1);
                    } else if r + 1 < row_count {
                        cur_row = Some(r + 1);
                        cur_col = Some(0);
                    }
                    cell_changed = true;
                }
            }
            Key::PageUp => {
                if let Some(r) = cur_row {
                    let page = if row_height > 0.0 {
                        (body_height / row_height).floor() as usize
                    } else {
                        0
                    };
                    let new_r = r.saturating_sub(page.max(1));
                    if new_r != r {
                        cur_row = Some(new_r);
                        cell_changed = true;
                    }
                }
            }
            Key::PageDown => {
                if let Some(r) = cur_row {
                    let page = if row_height > 0.0 {
                        (body_height / row_height).floor() as usize
                    } else {
                        0
                    };
                    let new_r = (r + page.max(1)).min(row_count - 1);
                    if new_r != r {
                        cur_row = Some(new_r);
                        cell_changed = true;
                    }
                }
            }
            Key::Enter => {
                activated = true;
            }
            Key::Escape => {
                core.input.focused_id = None;
                return DataGridKeyboardResponse::default();
            }
            _ => {}
        }
    }

    if let (Some(r), Some(c)) = (cur_row, cur_col) {
        let r = r.min(row_count - 1);
        let c = c.min(columns.len() - 1);
        state.has_active_cell = true;
        state.active_row_key = row_keys[r];
        state.active_column_key = columns[c].key;

        if cell_changed {
            let cell_x = col_layout.offsets[c];
            let cell_y = r as f32 * row_height;
            let cell_w = normalize_width(columns[c].width);
            let (new_sx, new_sy) = scroll_to_reveal(
                cell_x,
                cell_y,
                cell_w,
                row_height,
                viewport_w,
                body_height,
                state.scroll_x,
                state.scroll_y,
            );
            state.scroll_x = new_sx;
            state.scroll_y = new_sy;
        }
    }

    DataGridKeyboardResponse {
        activated,
        cell_changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use akar_core::{KeyEvent, Modifiers};
    use akar_layout::{length, Size, Style};

    fn make_grid_layout(w: f32, h: f32) -> (Layout, NodeId) {
        let mut layout = Layout::new();
        let node = layout.new_leaf(Style {
            size: Size {
                width: length(w),
                height: length(h),
            },
            ..Default::default()
        });
        let root = layout.new_with_children(Style::default(), &[node]);
        layout.compute(root, (Some(800.0), Some(600.0)), |_, _, _, _, _| Size::ZERO);
        (layout, node)
    }

    fn make_columns(widths: &[f32]) -> Vec<DataGridColumn> {
        widths
            .iter()
            .enumerate()
            .map(|(i, &w)| DataGridColumn {
                key: (i as u64 + 1) * 100,
                width: w,
                align: DataGridAlign::Left,
            })
            .collect()
    }

    fn make_row_keys(n: usize) -> Vec<u64> {
        (0..n as u64).map(|i| (i + 1) * 1000).collect()
    }

    fn default_style() -> DataGridStyle {
        DataGridStyle {
            header_bg: 0x1e293bff,
            header_text: 0xf8fafcff,
            row_bg: 0x09090bff,
            row_bg_alt: 0x18181bff,
            row_text: 0xfafafaff,
            selected_row_bg: 0x0f172aff,
            selected_row_text: 0xffffffff,
            active_cell_bg: 0x1e293bff,
            active_cell_text: 0xf8fafcff,
            hover_bg: 0x27272aff,
            grid_line_color: 0x27272aff,
            grid_line_width: 1.0,
            header_height: 36.0,
            row_height: 32.0,
            cell_padding_x: 12.0,
            cell_padding_y: 8.0,
            font_size: 16.0,
        }
    }

    // -- total_content_width / total_content_height --

    #[test]
    fn total_content_width_empty() {
        assert_eq!(total_content_width(&[]), 0.0);
    }

    #[test]
    fn total_content_width_sums_normalized() {
        let cols = make_columns(&[100.0, 200.0, 50.0]);
        assert_eq!(total_content_width(&cols), 350.0);
    }

    #[test]
    fn total_content_width_skips_non_finite() {
        let cols = vec![
            DataGridColumn {
                key: 1,
                width: f32::NAN,
                align: DataGridAlign::Left,
            },
            DataGridColumn {
                key: 2,
                width: 100.0,
                align: DataGridAlign::Left,
            },
            DataGridColumn {
                key: 3,
                width: f32::INFINITY,
                align: DataGridAlign::Left,
            },
        ];
        assert_eq!(total_content_width(&cols), 100.0);
    }

    #[test]
    fn total_content_height_zero_rows() {
        assert_eq!(total_content_height(0, 32.0), 0.0);
    }

    #[test]
    fn total_content_height_basic() {
        assert_eq!(total_content_height(100, 32.0), 3200.0);
    }

    // -- scroll_to_reveal --

    #[test]
    fn scroll_to_reveal_already_visible() {
        let (sx, sy) = scroll_to_reveal(50.0, 50.0, 100.0, 32.0, 400.0, 300.0, 0.0, 0.0);
        assert_eq!(sx, 0.0);
        assert_eq!(sy, 0.0);
    }

    #[test]
    fn scroll_to_reveal_cell_above() {
        let (sx, sy) = scroll_to_reveal(50.0, 10.0, 100.0, 32.0, 400.0, 300.0, 100.0, 100.0);
        // Cell x=50 is left of scroll_x=100, so sx scrolls to 50.0
        assert_eq!(sx, 50.0);
        // Cell y=10 is above scroll_y=100, so sy scrolls to 10.0
        assert_eq!(sy, 10.0);
    }

    #[test]
    fn scroll_to_reveal_cell_below() {
        let (sx, sy) = scroll_to_reveal(50.0, 500.0, 100.0, 32.0, 400.0, 300.0, 0.0, 0.0);
        assert_eq!(sx, 0.0);
        assert_eq!(sy, 500.0 + 32.0 - 300.0);
    }

    #[test]
    fn scroll_to_reveal_cell_left() {
        let (sx, sy) = scroll_to_reveal(10.0, 50.0, 100.0, 32.0, 400.0, 300.0, 200.0, 0.0);
        assert_eq!(sx, 10.0);
        assert_eq!(sy, 0.0);
    }

    #[test]
    fn scroll_to_reveal_cell_right() {
        let (sx, sy) = scroll_to_reveal(600.0, 50.0, 100.0, 32.0, 400.0, 300.0, 0.0, 0.0);
        assert_eq!(sx, 600.0 + 100.0 - 400.0);
        assert_eq!(sy, 0.0);
    }

    #[test]
    fn scroll_to_reveal_oversized_cell() {
        let (sx, sy) = scroll_to_reveal(0.0, 0.0, 800.0, 600.0, 400.0, 300.0, 100.0, 100.0);
        assert_eq!(sx, 0.0);
        assert_eq!(sy, 0.0);
    }

    #[test]
    fn scroll_to_reveal_clamps_to_zero() {
        let (sx, sy) = scroll_to_reveal(50.0, 50.0, 100.0, 32.0, 400.0, 300.0, 0.0, 0.0);
        assert!(sx >= 0.0);
        assert!(sy >= 0.0);
    }

    // -- compute_column_layout --

    #[test]
    fn normalized_column_layout_matches_widths() {
        let columns = make_columns(&[100.0, 200.0, 50.0]);
        let layout = compute_column_layout(&columns);
        assert_eq!(layout.offsets, vec![0.0, 100.0, 300.0, 350.0]);
        assert_eq!(layout.total_width, 350.0);
    }

    #[test]
    fn non_finite_column_width_normalized_to_zero() {
        let columns = vec![
            DataGridColumn {
                key: 1,
                width: f32::NAN,
                align: DataGridAlign::Left,
            },
            DataGridColumn {
                key: 2,
                width: 100.0,
                align: DataGridAlign::Left,
            },
        ];
        let layout = compute_column_layout(&columns);
        assert_eq!(layout.offsets, vec![0.0, 0.0, 100.0]);
    }

    #[test]
    fn column_layout_empty() {
        let layout = compute_column_layout(&[]);
        assert_eq!(layout.offsets, vec![0.0]);
        assert_eq!(layout.total_width, 0.0);
    }

    // -- visible_column_range --

    #[test]
    fn visible_columns_empty_offsets() {
        let range = visible_column_range(&[0.0], 0.0, 400.0, 0);
        assert_eq!(range, 0..0);
    }

    #[test]
    fn visible_columns_at_origin() {
        let offsets = vec![0.0, 100.0, 200.0, 300.0, 400.0, 500.0];
        let range = visible_column_range(&offsets, 0.0, 300.0, 5);
        assert!(range.start <= 2);
        assert!(range.end >= 3);
        assert!(range.end <= 7); // overscan can push past
    }

    #[test]
    fn visible_columns_scrolled() {
        let offsets = vec![0.0, 100.0, 200.0, 300.0, 400.0, 500.0];
        let range = visible_column_range(&offsets, 200.0, 300.0, 5);
        // Visible: columns 2..5 (200..500). With overscan: 0..5
        assert!(range.end >= 4);
        assert!(range.end <= 7);
    }

    #[test]
    fn visible_columns_with_overscan() {
        let offsets: Vec<f32> = (0..=100).map(|i| i as f32 * 100.0).collect();
        let range = visible_column_range(&offsets, 500.0, 300.0, 100);
        // Visible: columns 5..8 (500..800). With overscan 2: 3..10 or 3..11
        assert!(range.start <= 5);
        assert!(range.end >= 8);
        assert!(range.start >= 3);
        assert!(range.end <= 11);
    }

    // -- visible_row_range --

    #[test]
    fn visible_rows_empty() {
        let range = visible_row_range(0, 32.0, 0.0, 300.0);
        assert_eq!(range, 0..0);
    }

    #[test]
    fn visible_rows_zero_height() {
        let range = visible_row_range(10, 0.0, 0.0, 300.0);
        assert_eq!(range, 0..0);
    }

    #[test]
    fn visible_rows_covers_viewport() {
        let range = visible_row_range(100, 32.0, 0.0, 264.0);
        // 264/32 = 8.25, list_clip adds +1 each side, then overscan +2
        assert!(range.start == 0);
        assert!(range.end >= 9);
        assert!(range.end <= 15);
    }

    #[test]
    fn visible_rows_scrolled() {
        let range = visible_row_range(1000, 32.0, 3200.0, 264.0);
        // Row 100 at scroll 3200. list_clip gives ~99..109, overscan ~97..111
        assert!(range.start >= 95);
        assert!(range.end <= 115);
    }

    // -- Empty grid --

    #[test]
    fn empty_grid_returns_empty_ranges() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns: Vec<DataGridColumn> = vec![];
        let keys: Vec<u64> = vec![];
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 0, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        assert_eq!(resp.visible_rows, 0..0);
        assert_eq!(resp.visible_columns, 0..0);
        assert_eq!(resp.total_content_width, 0.0);
        assert_eq!(resp.total_content_height, 0.0);
    }

    // -- One cell --

    #[test]
    fn single_cell_grid() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0]);
        let keys = make_row_keys(1);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 1, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        assert_eq!(resp.visible_rows.start, 0);
        assert!(resp.visible_rows.end >= 1);
        assert_eq!(resp.visible_columns.start, 0);
        assert!(resp.visible_columns.end >= 1);
        assert_eq!(resp.total_content_width, 100.0);
        assert_eq!(resp.total_content_height, 32.0);
    }

    // -- Zero area viewport --

    #[test]
    fn zero_area_viewport_returns_empty() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(0.0, 0.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0, 100.0]);
        let keys = make_row_keys(10);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 10, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        assert_eq!(resp.visible_rows, 0..0);
    }

    // -- Scroll clamping --

    #[test]
    fn scroll_y_clamped_to_zero() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            scroll_y: -50.0,
            ..DataGridState::new()
        };
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();

        data_grid_begin(
            &mut core, &layout, node, &mut state, 10, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        assert_eq!(state.scroll_y, 0.0);
    }

    #[test]
    fn scroll_x_clamped_to_zero() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            scroll_x: -100.0,
            ..DataGridState::new()
        };
        let columns = make_columns(&[200.0; 5]);
        let keys = make_row_keys(10);
        let style = default_style();

        data_grid_begin(
            &mut core, &layout, node, &mut state, 10, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        assert_eq!(state.scroll_x, 0.0);
    }

    #[test]
    fn scroll_y_clamped_to_max() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            scroll_y: 99999.0,
            ..DataGridState::new()
        };
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();

        data_grid_begin(
            &mut core, &layout, node, &mut state, 10, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        let body_h: f32 = 300.0 - 36.0;
        let expected_max = (10.0f32 * 32.0 - body_h).max(0.0);
        assert_eq!(state.scroll_y, expected_max);
    }

    #[test]
    fn scroll_x_clamped_to_max() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            scroll_x: 99999.0,
            ..DataGridState::new()
        };
        let columns = make_columns(&[200.0; 5]);
        let keys = make_row_keys(10);
        let style = default_style();

        data_grid_begin(
            &mut core, &layout, node, &mut state, 10, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        let expected_max = (5.0f32 * 200.0 - 400.0).max(0.0);
        assert_eq!(state.scroll_x, expected_max);
    }

    // -- Active cell clearing --

    #[test]
    fn active_cell_cleared_when_key_absent() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            active_row_key: 9999,
            active_column_key: 100,
            has_active_cell: true,
            ..DataGridState::new()
        };
        let columns = make_columns(&[100.0; 3]);
        let keys = vec![1000u64, 2000, 3000];
        let style = default_style();

        data_grid_begin(
            &mut core, &layout, node, &mut state, 3, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        assert!(!state.has_active_cell);
    }

    #[test]
    fn active_cell_preserved_when_key_present() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            active_row_key: 2000,
            active_column_key: 200,
            has_active_cell: true,
            ..DataGridState::new()
        };
        let columns = make_columns(&[100.0; 3]);
        let keys = vec![1000u64, 2000, 3000];
        let style = default_style();

        data_grid_begin(
            &mut core, &layout, node, &mut state, 3, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        assert!(state.has_active_cell);
        assert_eq!(state.active_row_key, 2000);
    }

    // -- Scissor lifecycle --

    #[test]
    fn scissor_pushed_and_popped() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        assert!(core.draw_list.active_scissor().is_some());

        data_grid_header_begin(&mut core, &resp, &style);
        assert!(core.draw_list.active_scissor().is_some());
        data_grid_header_end(&mut core);

        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        assert!(core.draw_list.active_scissor().is_some());
        data_grid_body_end(&mut core);

        data_grid_end(&mut core);
        assert!(core.draw_list.active_scissor().is_none());
    }

    #[test]
    fn scrolled_body_rows_use_viewport_coordinates() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            scroll_y: 64.0,
            ..DataGridState::new()
        };
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(20);
        let style = default_style();

        let response = data_grid_begin(
            &mut core, &layout, node, &mut state, 20, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &response, &keys, &style, &[]);
        let cell = data_grid_cell(
            &mut core, &layout, &response, node, 2, keys[2], 0, &columns, &style, "row 2", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        assert_eq!(cell.rect, [0.0, 36.0, 100.0, 32.0]);
    }

    // -- Non-finite dimensions --

    #[test]
    fn non_finite_dimensions_normalized() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core,
            &layout,
            node,
            &mut state,
            5,
            &keys,
            f32::NAN,
            f32::INFINITY,
            &columns,
            &style,
        );
        data_grid_end(&mut core);

        assert!(resp.header_rect[3] > 0.0);
        assert!(resp.row_height > 0.0);
    }

    #[test]
    fn non_finite_column_widths_normalized() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = vec![
            DataGridColumn {
                key: 1,
                width: f32::NAN,
                align: DataGridAlign::Left,
            },
            DataGridColumn {
                key: 2,
                width: 100.0,
                align: DataGridAlign::Left,
            },
            DataGridColumn {
                key: 3,
                width: f32::INFINITY,
                align: DataGridAlign::Left,
            },
        ];
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        assert_eq!(resp.total_content_width, 100.0);
        assert_eq!(resp.visible_columns.end, 3); // 3 columns, all visible
    }

    // -- Visible ranges within bounds --

    #[test]
    fn visible_columns_within_bounds() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 10]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        assert!(resp.visible_columns.end <= 10);
    }

    #[test]
    fn visible_rows_cover_viewport() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(20);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 20, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        // Body height = 300 - 36 = 264. 264/32 = 8.25, so ~9 rows visible + overscan
        assert!(resp.visible_rows.len() >= 8);
        assert!(resp.visible_rows.end <= 20);
    }

    // -- Oversized cells --

    #[test]
    fn columns_wider_than_viewport_is_valid() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(200.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[500.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        assert_eq!(state.scroll_x, 0.0);
        assert!(resp.visible_columns.start <= 3);
    }

    // -- Exact boundaries --

    #[test]
    fn content_exactly_matches_viewport() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let rh = 32.0;
        let hh = 36.0;
        let rows = 8; // 8 * 32 = 256 body + 36 header = 292 < 300
        let vw = 400.0;
        let vh = 300.0;
        let (layout, node) = make_grid_layout(vw, vh);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 4]); // 400 total = viewport width
        let keys = make_row_keys(rows);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, rows, &keys, rh, hh, &columns, &style,
        );
        data_grid_end(&mut core);

        assert_eq!(state.scroll_x, 0.0);
        assert_eq!(state.scroll_y, 0.0);
        assert_eq!(resp.total_content_width, vw);
    }

    // -- Large scroll values --

    #[test]
    fn large_scroll_past_content_clamps() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            scroll_x: 1e9,
            scroll_y: 1e9,
            ..DataGridState::new()
        };
        let columns = make_columns(&[100.0; 5]);
        let keys = make_row_keys(10);
        let style = default_style();

        data_grid_begin(
            &mut core, &layout, node, &mut state, 10, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        let max_sx = (500.0f32 - 400.0).max(0.0);
        let max_sy = (320.0f32 - 264.0).max(0.0);
        assert_eq!(state.scroll_x, max_sx);
        assert_eq!(state.scroll_y, max_sy);
    }

    // -- Viewport resize --

    #[test]
    fn viewport_resize_changes_visible_range() {
        let columns = make_columns(&[100.0; 10]);
        let keys = make_row_keys(100);
        let style = default_style();

        let mut core1 = AkarCore::mock();
        core1.draw_list.begin_frame(1.0);
        let (layout1, node1) = make_grid_layout(400.0, 300.0);
        let mut state1 = DataGridState::new();
        let resp1 = data_grid_begin(
            &mut core1,
            &layout1,
            node1,
            &mut state1,
            100,
            &keys,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_end(&mut core1);

        let mut core2 = AkarCore::mock();
        core2.draw_list.begin_frame(1.0);
        let (layout2, node2) = make_grid_layout(200.0, 150.0);
        let mut state2 = DataGridState::new();
        let resp2 = data_grid_begin(
            &mut core2,
            &layout2,
            node2,
            &mut state2,
            100,
            &keys,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_end(&mut core2);

        assert!(resp1.visible_rows.len() > resp2.visible_rows.len());
        assert!(resp1.visible_columns.len() > resp2.visible_columns.len());
    }

    // -- Scroll consumption only when hovering --

    #[test]
    fn scroll_consumed_only_when_hovering() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let rect = layout.rect(node);

        // Not hovering
        core.draw_list.begin_frame(1.0);
        core.input.set_mouse_pos(rect[0] - 10.0, rect[1] + 10.0);
        core.input.push_scroll(0.0, -50.0);
        let mut state = DataGridState::new();
        data_grid_begin(
            &mut core, &layout, node, &mut state, 10, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);
        assert_eq!(state.scroll_y, 0.0);

        // Hovering
        core.input.begin_frame();
        core.draw_list.begin_frame(1.0);
        core.input.set_mouse_pos(rect[0] + 10.0, rect[1] + 10.0);
        core.input.push_scroll(0.0, -50.0);
        data_grid_begin(
            &mut core, &layout, node, &mut state, 10, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);
        assert_eq!(state.scroll_y, 50.0);
    }

    // -- Partial rows/columns (more than viewport but not huge) --

    #[test]
    fn partial_rows_and_columns() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 20]); // 2000 total width
        let keys = make_row_keys(50);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 50, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        assert!(resp.visible_columns.len() < 20);
        assert!(resp.visible_rows.len() < 50);
        assert!(resp.visible_columns.end <= 20);
        assert!(resp.visible_rows.end <= 50);
    }

    // -- 100,000 x 100 grid (bounded iteration) --

    #[test]
    fn large_grid_bounded_iteration() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 100]);
        let keys: Vec<u64> = (0..100_000).collect();
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 100_000, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        // Visible rows must be a small window, not 100k
        assert!(resp.visible_rows.len() < 100);
        assert!(resp.visible_rows.end <= 100_000);
        // Visible columns must be a small window, not 100
        assert!(resp.visible_columns.len() < 20);
        assert!(resp.visible_columns.end <= 100);
        // Content dimensions correct
        assert_eq!(resp.total_content_width, 10000.0);
        assert_eq!(resp.total_content_height, 3_200_000.0);
    }

    #[test]
    fn large_grid_with_scroll() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            scroll_y: 1_000_000.0,
            scroll_x: 5000.0,
            ..DataGridState::new()
        };
        let columns = make_columns(&[100.0; 100]);
        let keys: Vec<u64> = (0..100_000).collect();
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 100_000, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        // Clamped
        assert!(state.scroll_y < 3_200_000.0);
        assert!(state.scroll_x <= 10000.0 - 400.0);
        // Still bounded visible ranges
        assert!(resp.visible_rows.len() < 100);
        assert!(resp.visible_columns.len() < 20);
    }

    // -- Header/body x coordinates identical under scroll --

    #[test]
    fn header_and_body_x_coordinates_identical() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            scroll_x: 150.0,
            ..DataGridState::new()
        };
        let columns = make_columns(&[100.0; 10]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );

        // Get header cell rect for column 3
        let header_resp = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            3,
            &columns,
            &style,
            "test",
            DataGridSortDirection::None,
        );
        // Get body cell rect for column 3
        let cell_resp = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 3, &columns, &style, "test", false,
        );
        data_grid_end(&mut core);

        // Header and body x coordinates must be identical
        assert_eq!(header_resp.rect[0], cell_resp.rect[0]);
        assert_eq!(header_resp.rect[2], cell_resp.rect[2]);
    }

    // -- data_grid_cell uses correct row_height --

    #[test]
    fn cell_uses_row_height_not_body_height() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 10, &keys, 32.0, 36.0, &columns, &style,
        );

        let cell0 = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "a", false,
        );
        let cell1 = data_grid_cell(
            &mut core, &layout, &resp, node, 1, 2000, 0, &columns, &style, "b", false,
        );
        data_grid_end(&mut core);

        // Cell height should be row_height (32), not body height (264)
        assert_eq!(cell0.rect[3], 32.0);
        assert_eq!(cell1.rect[3], 32.0);
        // Cell y positions should be 32 apart
        assert_eq!(cell1.rect[1] - cell0.rect[1], 32.0);
    }

    // -- cell/column bounds checks --

    #[test]
    fn header_cell_out_of_bounds_returns_zero() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        let hr = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            99,
            &columns,
            &style,
            "x",
            DataGridSortDirection::None,
        );
        data_grid_end(&mut core);

        assert_eq!(hr.rect, [0.0; 4]);
    }

    #[test]
    fn body_cell_out_of_bounds_returns_zero() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        let cr = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 99, &columns, &style, "x", false,
        );
        data_grid_end(&mut core);

        assert_eq!(cr.rect, [0.0; 4]);
    }

    // -- Content dimensions in response --

    #[test]
    fn response_content_dimensions() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0, 200.0, 50.0]);
        let keys = make_row_keys(50);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 50, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        assert_eq!(resp.total_content_width, 350.0);
        assert_eq!(resp.total_content_height, 50.0 * 32.0);
        assert_eq!(resp.row_height, 32.0);
        assert_eq!(resp.scroll_x, 0.0);
    }

    // -- Header/body rect correctness --

    #[test]
    fn header_and_body_rects_partition_viewport() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        // Header starts at viewport top
        assert_eq!(resp.header_rect[1], resp.viewport_rect[1]);
        // Body starts right below header
        assert_eq!(resp.body_rect[1], resp.header_rect[1] + resp.header_rect[3]);
        // Same x and width
        assert_eq!(resp.header_rect[0], resp.body_rect[0]);
        assert_eq!(resp.header_rect[2], resp.body_rect[2]);
        // Heights sum to viewport height
        let total_h = resp.header_rect[3] + resp.body_rect[3];
        assert!((total_h - resp.viewport_rect[3]).abs() < 1.0);
    }

    // -- Task 3: Styling and scissor tests --

    #[test]
    fn outer_surface_quad_submitted() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_end(&mut core);

        let quads = core.draw_list.sorted_quads();
        assert!(!quads.is_empty());
        let outer = &quads[0];
        assert_eq!(outer.fill, color_to_f32(style.row_bg));
    }

    #[test]
    fn header_background_quad_submitted() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        let count_before = core.draw_list.len();
        data_grid_header_begin(&mut core, &resp, &style);
        let count_after = core.draw_list.len();
        data_grid_header_end(&mut core);
        data_grid_end(&mut core);

        assert!(count_after > count_before);
        let quads = core.draw_list.sorted_quads();
        let header_quad = quads
            .iter()
            .find(|q| q.fill == color_to_f32(style.header_bg) && q.rect[3] == 36.0);
        assert!(header_quad.is_some());
    }

    #[test]
    fn body_alternating_row_backgrounds() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(20);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 20, &keys, 32.0, 36.0, &columns, &style,
        );
        let count_before = core.draw_list.len();
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        let count_after = core.draw_list.len();
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        assert!(count_after > count_before);
        let quads = core.draw_list.sorted_quads();
        let row_bg_even = color_to_f32(style.row_bg);
        let row_bg_odd = color_to_f32(style.row_bg_alt);
        let has_even = quads
            .iter()
            .any(|q| q.fill == row_bg_even && q.rect[1] >= resp.body_rect[1]);
        let has_odd = quads
            .iter()
            .any(|q| q.fill == row_bg_odd && q.rect[1] >= resp.body_rect[1]);
        assert!(has_even);
        assert!(has_odd);
    }

    #[test]
    fn grid_lines_submitted() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(20);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 20, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let gl = color_to_f32(style.grid_line_color);
        let quads = core.draw_list.sorted_quads();
        let grid_line_quads: Vec<_> = quads.iter().filter(|q| q.fill == gl).collect();
        assert!(!grid_line_quads.is_empty());
    }

    #[test]
    fn active_cell_highlight_submitted() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 2000,
            active_column_key: 200,
            ..DataGridState::new()
        };
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let active_bg = color_to_f32(style.active_cell_bg);
        let quads = core.draw_list.sorted_quads();
        let active_quad = quads.iter().find(|q| q.fill == active_bg);
        assert!(active_quad.is_some());
    }

    #[test]
    fn selected_row_highlight_submitted() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();
        let selected = vec![2000u64];

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        let count_before = core.draw_list.len();
        data_grid_body_begin(&mut core, &resp, &keys, &style, &selected);
        let count_after = core.draw_list.len();
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        assert!(count_after > count_before);
        let sel_bg = color_to_f32(style.selected_row_bg);
        let quads = core.draw_list.sorted_quads();
        let sel_quad = quads.iter().find(|q| q.fill == sel_bg);
        assert!(sel_quad.is_some());
    }

    #[test]
    fn body_scissor_excludes_header() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(20);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 20, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        let body_scissor = core.draw_list.active_scissor().unwrap();
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let header_bottom = resp.header_rect[1] + resp.header_rect[3];
        let body_top = resp.body_rect[1];
        assert!((header_bottom - body_top).abs() < 0.01);
        assert!(body_scissor[1] >= header_bottom - 0.01);
    }

    #[test]
    fn zero_area_viewport_no_panic() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(0.0, 0.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_header_begin(&mut core, &resp, &style);
        data_grid_header_end(&mut core);
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);
    }

    #[test]
    fn nested_scissor_intersection_preserved() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        core.draw_list.push_scissor([50.0, 50.0, 200.0, 200.0]);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_header_begin(&mut core, &resp, &style);
        let header_scissor = core.draw_list.active_scissor().unwrap();
        data_grid_header_end(&mut core);

        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        let body_scissor = core.draw_list.active_scissor().unwrap();
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);
        core.draw_list.pop_scissor();

        assert!(header_scissor[0] >= 50.0);
        assert!(header_scissor[1] >= 50.0);
        assert!(body_scissor[0] >= 50.0);
        assert!(body_scissor[1] >= 50.0);
    }

    #[test]
    fn quad_count_proportional_to_visible_rows() {
        let mut core_small = AkarCore::mock();
        core_small.draw_list.begin_frame(1.0);
        let (layout_s, node_s) = make_grid_layout(400.0, 100.0);
        let mut state_s = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(1000);
        let style = default_style();

        let resp_s = data_grid_begin(
            &mut core_small,
            &layout_s,
            node_s,
            &mut state_s,
            1000,
            &keys,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_body_begin(&mut core_small, &resp_s, &keys, &style, &[]);
        data_grid_body_end(&mut core_small);
        data_grid_end(&mut core_small);
        let small_count = core_small.draw_list.len();

        let mut core_large = AkarCore::mock();
        core_large.draw_list.begin_frame(1.0);
        let (layout_l, node_l) = make_grid_layout(400.0, 600.0);
        let mut state_l = DataGridState::new();

        let resp_l = data_grid_begin(
            &mut core_large,
            &layout_l,
            node_l,
            &mut state_l,
            1000,
            &keys,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_body_begin(&mut core_large, &resp_l, &keys, &style, &[]);
        data_grid_body_end(&mut core_large);
        data_grid_end(&mut core_large);
        let large_count = core_large.draw_list.len();

        assert!(large_count > small_count);
        assert!(resp_l.visible_rows.len() > resp_s.visible_rows.len());
    }

    #[test]
    fn post_scroll_reflected_in_drawing() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let rect = layout.rect(node);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(100);
        let style = default_style();

        core.input.set_mouse_pos(rect[0] + 10.0, rect[1] + 10.0);
        core.input.push_scroll(0.0, -200.0);
        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 100, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        assert_eq!(state.scroll_y, 200.0);
        assert!(resp.visible_rows.start > 0);
    }

    // -- Task 4: Text rendering tests --

    fn text_calls(draw_list: &akar_core::DrawList) -> Vec<TextCall> {
        draw_list
            .text_calls()
            .iter()
            .filter_map(|c| match c {
                akar_core::DrawCall::Text(t) => Some(t.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn body_cell_id_stable_across_scroll() {
        let columns = make_columns(&[100.0; 3]);
        let style = default_style();
        let keys = make_row_keys(100);

        let mut core0 = AkarCore::mock();
        core0.draw_list.begin_frame(1.0);
        let (layout0, node0) = make_grid_layout(400.0, 300.0);
        let mut state0 = DataGridState::new();
        let resp0 = data_grid_begin(
            &mut core0,
            &layout0,
            node0,
            &mut state0,
            100,
            &keys,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_body_begin(&mut core0, &resp0, &keys, &style, &[]);
        let _ = data_grid_cell(
            &mut core0, &layout0, &resp0, node0, 1, 1001, 0, &columns, &style, "row1", false,
        );
        data_grid_body_end(&mut core0);
        let tc0 = text_calls(&core0.draw_list);
        let id_before = tc0[0].buffer_id;
        data_grid_end(&mut core0);

        let mut core1 = AkarCore::mock();
        core1.draw_list.begin_frame(1.0);
        let (layout1, node1) = make_grid_layout(400.0, 300.0);
        let mut state1 = DataGridState {
            scroll_y: 32.0,
            ..DataGridState::new()
        };
        let resp1 = data_grid_begin(
            &mut core1,
            &layout1,
            node1,
            &mut state1,
            100,
            &keys,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_body_begin(&mut core1, &resp1, &keys, &style, &[]);
        let _ = data_grid_cell(
            &mut core1, &layout1, &resp1, node1, 1, 1001, 0, &columns, &style, "row1", false,
        );
        data_grid_body_end(&mut core1);
        let tc1 = text_calls(&core1.draw_list);
        let id_after = tc1[0].buffer_id;
        data_grid_end(&mut core1);

        assert_eq!(id_before, id_after);
    }

    #[test]
    fn header_and_body_ids_in_separate_domains() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_header_begin(&mut core, &resp, &style);
        let _ = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            0,
            &columns,
            &style,
            "Name",
            DataGridSortDirection::None,
        );
        data_grid_header_end(&mut core);

        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        let _ = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "Alice", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let texts = text_calls(&core.draw_list);
        assert!(texts.len() >= 2);
        let hdr_id = texts[0].buffer_id;
        let body_id = texts[1].buffer_id;
        assert_ne!(hdr_id, body_id);
        assert!(hdr_id & HEADER_DOMAIN_OFFSET != 0);
        assert!(body_id & HEADER_DOMAIN_OFFSET == 0);
    }

    #[test]
    fn body_cells_differ_across_rows_and_columns() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        let _ = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "r0c0", false,
        );
        let _ = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 1, &columns, &style, "r0c1", false,
        );
        let _ = data_grid_cell(
            &mut core, &layout, &resp, node, 1, 2000, 0, &columns, &style, "r1c0", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let texts = text_calls(&core.draw_list);
        assert_eq!(texts.len(), 3);
        assert_ne!(texts[0].buffer_id, texts[1].buffer_id);
        assert_ne!(texts[0].buffer_id, texts[2].buffer_id);
        assert_ne!(texts[1].buffer_id, texts[2].buffer_id);
    }

    #[test]
    fn large_grid_shapes_only_visible_cells() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10_000);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 10_000, &keys, 32.0, 36.0, &columns, &style,
        );
        let visible = resp.visible_rows.clone();
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        for row_i in visible.clone() {
            for col_i in 0..columns.len() {
                let _ = data_grid_cell(
                    &mut core,
                    &layout,
                    &resp,
                    node,
                    row_i,
                    keys[row_i],
                    col_i,
                    &columns,
                    &style,
                    "cell",
                    false,
                );
            }
        }
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let texts = text_calls(&core.draw_list);
        // Overscan cells outside the body scissor are culled, so count <= visible * cols
        assert!(texts.len() <= visible.len() * columns.len());
        // But we still shape text for all in-range visible cells
        assert!(texts.len() >= (visible.len() - 4) * columns.len());
        assert!(texts.len() < 10_000 * columns.len());
    }

    #[test]
    fn long_text_clipped_to_cell() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        let long_text = "This is a very long text that should be clipped to the cell boundary and never bleed into adjacent columns";
        let cell_resp = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, long_text, false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let texts = text_calls(&core.draw_list);
        assert_eq!(texts.len(), 1);
        let tc = &texts[0];
        assert_eq!(tc.clip[0], cell_resp.rect[0]);
        assert_eq!(tc.clip[2], cell_resp.rect[2]);
    }

    #[test]
    fn empty_text_submits_no_draw_call() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        let _ = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let texts = text_calls(&core.draw_list);
        assert!(texts.is_empty());
    }

    #[test]
    fn sort_indicator_submitted_for_ascending() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_header_begin(&mut core, &resp, &style);
        let _ = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            0,
            &columns,
            &style,
            "Name",
            DataGridSortDirection::Ascending,
        );
        data_grid_header_end(&mut core);
        data_grid_end(&mut core);

        let texts = text_calls(&core.draw_list);
        assert_eq!(texts.len(), 2);
    }

    #[test]
    fn sort_indicator_submitted_for_descending() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_header_begin(&mut core, &resp, &style);
        let _ = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            0,
            &columns,
            &style,
            "Name",
            DataGridSortDirection::Descending,
        );
        data_grid_header_end(&mut core);
        data_grid_end(&mut core);

        let texts = text_calls(&core.draw_list);
        assert_eq!(texts.len(), 2);
    }

    #[test]
    fn no_sort_indicator_for_none() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_header_begin(&mut core, &resp, &style);
        let _ = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            0,
            &columns,
            &style,
            "Name",
            DataGridSortDirection::None,
        );
        data_grid_header_end(&mut core);
        data_grid_end(&mut core);

        let texts = text_calls(&core.draw_list);
        assert_eq!(texts.len(), 1);
    }

    #[test]
    fn empty_header_label_submits_no_text() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_header_begin(&mut core, &resp, &style);
        let _ = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            0,
            &columns,
            &style,
            "",
            DataGridSortDirection::None,
        );
        data_grid_header_end(&mut core);
        data_grid_end(&mut core);

        let texts = text_calls(&core.draw_list);
        assert!(texts.is_empty());
    }

    #[test]
    fn text_z_is_above_base() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        let _ = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "hello", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let texts = text_calls(&core.draw_list);
        assert_eq!(texts.len(), 1);
        assert!(texts[0].z > Z_BASE);
    }

    #[test]
    fn selected_row_uses_selected_text_color() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        let _ = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "sel", true,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let texts = text_calls(&core.draw_list);
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].color, color_to_f32(style.selected_row_text));
    }

    #[test]
    fn normal_row_uses_row_text_color() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        let _ = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "norm", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let texts = text_calls(&core.draw_list);
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].color, color_to_f32(style.row_text));
    }

    // -- Task 5: Pointer interaction and selection intent --

    fn simulate_click(core: &mut AkarCore, x: f32, y: f32) {
        core.input.set_mouse_pos(x, y);
        core.input.push_mouse_button(0, true);
        core.input.begin_frame();
        core.input.set_mouse_pos(x, y);
        core.input.push_mouse_button(0, false);
    }

    #[test]
    fn body_cell_click_sets_activated_keys() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        let cell_resp = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "r0c0", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        assert_eq!(cell_resp.row_key, 1000);
        assert_eq!(cell_resp.column_key, 100);
        assert_eq!(cell_resp.row_index, 0);
        assert_eq!(cell_resp.column_index, 0);
    }

    #[test]
    fn body_cell_click_detected_with_keys() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        core.draw_list.begin_frame(1.0);
        let resp = data_grid_begin(
            &mut core,
            &layout,
            node,
            &mut DataGridState::new(),
            5,
            &keys,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);

        let cell_rect = {
            let [vx, _, _, _] = resp.viewport_rect;
            let [_, body_y, _, _] = resp.body_rect;
            [vx, body_y, 100.0, 32.0]
        };
        simulate_click(&mut core, cell_rect[0] + 10.0, cell_rect[1] + 10.0);

        let cell_resp = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "r0c0", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        assert!(cell_resp.clicked);
        assert_eq!(cell_resp.row_key, 1000);
        assert_eq!(cell_resp.column_key, 100);
    }

    #[test]
    fn header_cell_click_sets_column_key() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        core.draw_list.begin_frame(1.0);
        let resp = data_grid_begin(
            &mut core,
            &layout,
            node,
            &mut DataGridState::new(),
            5,
            &keys,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_header_begin(&mut core, &resp, &style);

        simulate_click(
            &mut core,
            resp.viewport_rect[0] + 10.0,
            resp.viewport_rect[1] + 10.0,
        );

        let hdr_resp = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            0,
            &columns,
            &style,
            "Name",
            DataGridSortDirection::None,
        );
        data_grid_header_end(&mut core);
        data_grid_end(&mut core);

        assert!(hdr_resp.clicked);
        assert_eq!(hdr_resp.column_key, 100);
    }

    #[test]
    fn click_outside_grid_body_has_no_effect() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core,
            &layout,
            node,
            &mut DataGridState::new(),
            5,
            &keys,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);

        simulate_click(&mut core, 500.0, 500.0);

        let cell_resp = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "r0c0", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        assert!(!cell_resp.clicked);
        assert!(!cell_resp.hovered);
    }

    #[test]
    fn partially_visible_cell_clickable_within_scissor() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[200.0; 5]);
        let keys = make_row_keys(5);
        let style = default_style();

        state.scroll_x = 150.0;
        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);

        let cell_resp = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 1, &columns, &style, "c1", false,
        );
        let cell_rect = cell_resp.rect;

        let scissor = core.draw_list.active_scissor().unwrap();
        let clipped = rect_intersect(cell_rect, scissor);

        simulate_click(&mut core, clipped[0] + 1.0, clipped[1] + 1.0);

        let cell_resp2 = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 1, &columns, &style, "c1", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        assert!(cell_resp2.clicked);
    }

    #[test]
    fn cell_outside_scissor_not_clickable() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[200.0; 5]);
        let keys = make_row_keys(5);
        let style = default_style();

        state.scroll_x = 150.0;
        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);

        let cell_resp = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 1, &columns, &style, "c1", false,
        );
        let cell_rect = cell_resp.rect;

        simulate_click(
            &mut core,
            cell_rect[0] + cell_rect[2] + 5.0,
            cell_rect[1] + 5.0,
        );

        let cell_resp2 = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 1, &columns, &style, "c1", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        assert!(!cell_resp2.clicked);
    }

    #[test]
    fn grid_receives_focus_on_cell_click() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        core.draw_list.begin_frame(1.0);
        let resp = data_grid_begin(
            &mut core,
            &layout,
            node,
            &mut DataGridState::new(),
            5,
            &keys,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);

        let [vx, _, _, _] = resp.viewport_rect;
        let [_, body_y, _, _] = resp.body_rect;
        simulate_click(&mut core, vx + 10.0, body_y + 10.0);

        let _ = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "r0c0", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let expected_id = layout.widget_id(node);
        assert_eq!(core.input.focused_id, Some(expected_id));
    }

    #[test]
    fn cell_click_after_sort_reports_correct_keys() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let style = default_style();

        let keys_before = vec![1000u64, 2000, 3000];
        core.draw_list.begin_frame(1.0);
        let resp = data_grid_begin(
            &mut core,
            &layout,
            node,
            &mut DataGridState::new(),
            3,
            &keys_before,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys_before, &style, &[]);
        let cell_before = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "first", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let keys_after = vec![3000u64, 2000, 1000];
        core.input.begin_frame();
        core.draw_list.begin_frame(1.0);
        let resp2 = data_grid_begin(
            &mut core,
            &layout,
            node,
            &mut DataGridState::new(),
            3,
            &keys_after,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_body_begin(&mut core, &resp2, &keys_after, &style, &[]);
        let cell_after = data_grid_cell(
            &mut core, &layout, &resp2, node, 0, 3000, 0, &columns, &style, "third", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        assert_eq!(cell_before.row_key, 1000);
        assert_eq!(cell_after.row_key, 3000);
        assert_eq!(cell_before.column_key, cell_after.column_key);
    }

    #[test]
    fn hover_highlight_submitted_for_hovered_cell() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        core.draw_list.begin_frame(1.0);
        let resp = data_grid_begin(
            &mut core,
            &layout,
            node,
            &mut DataGridState::new(),
            5,
            &keys,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);

        let [vx, _, _, _] = resp.viewport_rect;
        let [_, body_y, _, _] = resp.body_rect;
        core.input.set_mouse_pos(vx + 10.0, body_y + 10.0);

        let cell_resp = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "hover", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let hover_bg = color_to_f32(style.hover_bg);
        let quads = core.draw_list.sorted_quads();
        let hover_quad = quads
            .iter()
            .find(|q| q.fill == hover_bg && q.z == Z_BASE + 0.005 && q.rect == cell_resp.rect);
        assert!(hover_quad.is_some());
    }

    #[test]
    fn no_hover_highlight_when_not_hovering() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        core.draw_list.begin_frame(1.0);
        let resp = data_grid_begin(
            &mut core,
            &layout,
            node,
            &mut DataGridState::new(),
            5,
            &keys,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);

        core.input.set_mouse_pos(500.0, 500.0);

        let cell_resp = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "nope", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        assert!(!cell_resp.hovered);
        let hover_z = Z_BASE + 0.005;
        let quads = core.draw_list.sorted_quads();
        let hover_quad = quads
            .iter()
            .find(|q| q.z == hover_z && q.rect == cell_resp.rect);
        assert!(hover_quad.is_none());
    }

    #[test]
    fn visual_precedence_active_over_selected_over_hover() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 100,
            ..DataGridState::new()
        };
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        core.draw_list.begin_frame(1.0);
        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[1000]);

        let [vx, _, _, _] = resp.viewport_rect;
        let [_, body_y, _, _] = resp.body_rect;
        core.input.set_mouse_pos(vx + 10.0, body_y + 10.0);

        let _ = data_grid_cell(
            &mut core, &layout, &resp, node, 0, 1000, 0, &columns, &style, "cell", true,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        let hover_bg = color_to_f32(style.hover_bg);
        let selected_bg = color_to_f32(style.selected_row_bg);
        let active_bg = color_to_f32(style.active_cell_bg);
        let quads = core.draw_list.sorted_quads();

        let hover_z = quads.iter().find(|q| q.fill == hover_bg).map(|q| q.z);
        let selected_z = quads.iter().find(|q| q.fill == selected_bg).map(|q| q.z);
        let active_z = quads.iter().find(|q| q.fill == active_bg).map(|q| q.z);

        assert!(hover_z.is_some());
        assert!(selected_z.is_some());
        assert!(active_z.is_some());
        assert!(hover_z.unwrap() < selected_z.unwrap());
        assert!(selected_z.unwrap() < active_z.unwrap());
    }

    #[test]
    fn rect_intersect_overlapping() {
        let a = [10.0, 10.0, 100.0, 100.0];
        let b = [50.0, 50.0, 100.0, 100.0];
        let r = rect_intersect(a, b);
        assert_eq!(r, [50.0, 50.0, 60.0, 60.0]);
    }

    #[test]
    fn rect_intersect_no_overlap() {
        let a = [0.0, 0.0, 10.0, 10.0];
        let b = [50.0, 50.0, 10.0, 10.0];
        let r = rect_intersect(a, b);
        assert_eq!(r, [50.0, 50.0, 0.0, 0.0]);
    }

    #[test]
    fn rect_intersect_contained() {
        let a = [0.0, 0.0, 100.0, 100.0];
        let b = [20.0, 20.0, 30.0, 30.0];
        let r = rect_intersect(a, b);
        assert_eq!(r, [20.0, 20.0, 30.0, 30.0]);
    }

    #[test]
    fn header_cell_response_has_column_key() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0, 200.0, 50.0]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_header_begin(&mut core, &resp, &style);
        let h0 = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            0,
            &columns,
            &style,
            "A",
            DataGridSortDirection::None,
        );
        let h1 = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            1,
            &columns,
            &style,
            "B",
            DataGridSortDirection::None,
        );
        let h2 = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            2,
            &columns,
            &style,
            "C",
            DataGridSortDirection::None,
        );
        data_grid_header_end(&mut core);
        data_grid_end(&mut core);

        assert_eq!(h0.column_key, 100);
        assert_eq!(h1.column_key, 200);
        assert_eq!(h2.column_key, 300);
    }

    #[test]
    fn body_cell_response_has_all_keys() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState::new();
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_body_begin(&mut core, &resp, &keys, &style, &[]);
        let c1 = data_grid_cell(
            &mut core, &layout, &resp, node, 2, 3000, 1, &columns, &style, "r2c1", false,
        );
        data_grid_body_end(&mut core);
        data_grid_end(&mut core);

        assert_eq!(c1.row_key, 3000);
        assert_eq!(c1.column_key, 200);
        assert_eq!(c1.row_index, 2);
        assert_eq!(c1.column_index, 1);
    }

    #[test]
    fn header_scissor_blocks_off_screen_click() {
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let mut state = DataGridState {
            scroll_x: 500.0,
            ..DataGridState::new()
        };
        let columns = make_columns(&[200.0; 5]);
        let keys = make_row_keys(5);
        let style = default_style();

        let resp = data_grid_begin(
            &mut core, &layout, node, &mut state, 5, &keys, 32.0, 36.0, &columns, &style,
        );
        data_grid_header_begin(&mut core, &resp, &style);

        let hdr_resp = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            0,
            &columns,
            &style,
            "Col0",
            DataGridSortDirection::None,
        );
        let cell_rect = hdr_resp.rect;

        simulate_click(&mut core, cell_rect[0] + 5.0, cell_rect[1] + 5.0);

        let hdr_resp2 = data_grid_header_cell(
            &mut core,
            &layout,
            &resp,
            node,
            0,
            &columns,
            &style,
            "Col0",
            DataGridSortDirection::None,
        );
        data_grid_header_end(&mut core);
        data_grid_end(&mut core);

        assert!(!hdr_resp2.clicked);
    }

    // -- Task 6: Keyboard navigation tests --

    fn grid_focus_id(layout: &Layout, node: NodeId) -> u64 {
        layout.widget_id_keyed(node, 0)
    }

    fn set_grid_focus(core: &mut AkarCore, layout: &Layout, node: NodeId) {
        core.input.focused_id = Some(grid_focus_id(layout, node));
    }

    #[test]
    fn arrow_down_moves_active_cell() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Down);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert!(!resp.activated);
        assert_eq!(state.active_row_key, 2000);
        assert_eq!(state.active_column_key, 100);
    }

    #[test]
    fn arrow_up_moves_active_cell() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 3000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Up);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_row_key, 2000);
    }

    #[test]
    fn arrow_left_moves_active_cell() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 200,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Left);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_column_key, 100);
    }

    #[test]
    fn arrow_right_moves_active_cell() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Right);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_column_key, 200);
    }

    #[test]
    fn home_moves_to_first_column() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 5000,
            active_column_key: 300,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Home);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_column_key, 100);
        assert_eq!(state.active_row_key, 5000);
    }

    #[test]
    fn end_moves_to_last_column() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 5000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::End);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_column_key, 300);
        assert_eq!(state.active_row_key, 5000);
    }

    #[test]
    fn ctrl_home_moves_to_first_row() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 5000,
            active_column_key: 200,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.modifiers = Modifiers {
            control: true,
            ..Modifiers::default()
        };
        core.input.push_key(Key::Home);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_row_key, 1000);
        assert_eq!(state.active_column_key, 200);
    }

    #[test]
    fn ctrl_end_moves_to_last_row() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 200,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.modifiers = Modifiers {
            control: true,
            ..Modifiers::default()
        };
        core.input.push_key(Key::End);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_row_key, 10000);
        assert_eq!(state.active_column_key, 200);
    }

    #[test]
    fn page_down_moves_by_visible_rows() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(100);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::PageDown);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 100, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        let expected_page = (264.0_f32 / 32.0).floor() as usize;
        let expected_row_key = (expected_page as u64 + 1) * 1000;
        assert_eq!(state.active_row_key, expected_row_key);
    }

    #[test]
    fn page_up_moves_by_visible_rows() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(100);
        let style = default_style();
        let page = (264.0_f32 / 32.0).floor() as usize;
        let start_row = page + 5;
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: ((start_row as u64) + 1) * 1000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::PageUp);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 100, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        let expected_idx = start_row - page;
        let expected_key = ((expected_idx as u64) + 1) * 1000;
        assert_eq!(state.active_row_key, expected_key);
    }

    #[test]
    fn tab_moves_to_next_column() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Tab);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_column_key, 200);
        assert_eq!(state.active_row_key, 1000);
    }

    #[test]
    fn tab_wraps_to_next_row() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 300,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Tab);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_row_key, 2000);
        assert_eq!(state.active_column_key, 100);
    }

    #[test]
    fn shift_tab_moves_to_previous_column() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 200,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key_event(KeyEvent {
            key: Key::Tab,
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            repeat: false,
        });

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_column_key, 100);
        assert_eq!(state.active_row_key, 1000);
    }

    #[test]
    fn shift_tab_wraps_to_previous_row() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 2000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key_event(KeyEvent {
            key: Key::Tab,
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            repeat: false,
        });

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_row_key, 1000);
        assert_eq!(state.active_column_key, 300);
    }

    #[test]
    fn navigation_clamps_at_top_left() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Up);
        core.input.push_key(Key::Left);

        let _resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert_eq!(state.active_row_key, 1000);
        assert_eq!(state.active_column_key, 100);
    }

    #[test]
    fn navigation_clamps_at_bottom_right() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 10000,
            active_column_key: 300,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Down);
        core.input.push_key(Key::Right);

        let _resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert_eq!(state.active_row_key, 10000);
        assert_eq!(state.active_column_key, 300);
    }

    #[test]
    fn empty_grid_navigation_does_nothing() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns: Vec<DataGridColumn> = vec![];
        let keys: Vec<u64> = vec![];
        let style = default_style();
        let mut state = DataGridState::new();

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Down);
        core.input.push_key(Key::Right);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 0, &keys, &columns, &style,
        );

        assert_eq!(resp, DataGridKeyboardResponse::default());
        assert!(!state.has_active_cell);
    }

    #[test]
    fn row_count_shrink_clamps_active_cell() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = vec![5000u64, 6000, 7000];
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 10000,
            active_column_key: 300,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Down);

        let _resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 3, &keys, &columns, &style,
        );

        assert_eq!(state.active_row_key, 7000);
        assert_eq!(state.active_column_key, 300);
    }

    #[test]
    fn enter_returns_activated() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 2000,
            active_column_key: 200,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Enter);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.activated);
    }

    #[test]
    fn ignored_when_another_widget_has_focus() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 2000,
            active_column_key: 200,
            ..DataGridState::new()
        };

        core.input.focused_id = Some(999999);
        core.input.push_key(Key::Down);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert_eq!(resp, DataGridKeyboardResponse::default());
        assert_eq!(state.active_row_key, 2000);
        assert_eq!(state.active_column_key, 200);
    }

    #[test]
    fn scroll_to_reveal_after_down_movement() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 128.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(100);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 100,
            scroll_y: 0.0,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Down);

        data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 100, &keys, &columns, &style,
        );

        assert!(state.scroll_y >= 0.0);
    }

    #[test]
    fn scroll_to_reveal_scrolls_down_when_needed() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 128.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(100);
        let style = default_style();
        let body_height: f32 = 128.0 - 36.0;
        let page = (body_height / 32.0_f32).floor() as usize;
        let start_key = ((page as u64) + 2) * 1000;
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: start_key,
            active_column_key: 100,
            scroll_y: 0.0,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Down);

        data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 100, &keys, &columns, &style,
        );

        assert!(state.scroll_y > 0.0);
    }

    #[test]
    fn sort_preserves_active_cell_by_key() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 2000,
            active_column_key: 200,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Up);

        let keys_after = vec![3000u64, 1000, 2000];
        let resp = data_grid_handle_keyboard(
            &mut core,
            &layout,
            node,
            &mut state,
            3,
            &keys_after,
            &columns,
            &style,
        );

        assert!(resp.cell_changed);
        let idx = keys_after
            .iter()
            .position(|&k| k == state.active_row_key)
            .unwrap();
        assert_eq!(idx, 1);
        assert_eq!(state.active_row_key, 1000);
    }

    #[test]
    fn arrow_key_initializes_active_cell_when_none() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState::new();

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Down);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert!(state.has_active_cell);
        assert_eq!(state.active_row_key, 2000);
        assert_eq!(state.active_column_key, 100);
    }

    #[test]
    fn escape_releases_focus() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 2000,
            active_column_key: 200,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Escape);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert_eq!(resp, DataGridKeyboardResponse::default());
        assert_eq!(core.input.focused_id, None);
    }

    #[test]
    fn page_down_clamps_at_last_row() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 600.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 8000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::PageDown);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_row_key, 10000);
    }

    #[test]
    fn page_up_clamps_at_first_row() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 600.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::PageUp);

        let _resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert_eq!(state.active_row_key, 1000);
    }

    #[test]
    fn tab_on_last_cell_does_not_wrap_past_end() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(3);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 3000,
            active_column_key: 300,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Tab);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 3, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_row_key, 3000);
        assert_eq!(state.active_column_key, 300);
    }

    #[test]
    fn shift_tab_on_first_cell_does_not_wrap_past_start() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(3);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key_event(KeyEvent {
            key: Key::Tab,
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            repeat: false,
        });

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 3, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_row_key, 1000);
        assert_eq!(state.active_column_key, 100);
    }

    #[test]
    fn no_keys_returns_empty_response() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 2000,
            active_column_key: 200,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert_eq!(resp, DataGridKeyboardResponse::default());
        assert_eq!(state.active_row_key, 2000);
        assert_eq!(state.active_column_key, 200);
    }

    #[test]
    fn active_key_absent_from_row_keys_clamps() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = vec![5000u64, 6000, 7000];
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 2000,
            active_column_key: 200,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Down);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 3, &keys, &columns, &style,
        );

        assert_eq!(resp, DataGridKeyboardResponse::default());
    }

    #[test]
    fn multiple_navigation_keys_processed_sequentially() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0; 3]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 1000,
            active_column_key: 100,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Down);
        core.input.push_key(Key::Right);
        core.input.push_key(Key::Down);

        let resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert!(resp.cell_changed);
        assert_eq!(state.active_row_key, 3000);
        assert_eq!(state.active_column_key, 200);
    }

    #[test]
    fn column_count_shrink_clamps_active_cell() {
        let mut core = AkarCore::mock();
        let (layout, node) = make_grid_layout(400.0, 300.0);
        let columns = make_columns(&[100.0, 100.0]);
        let keys = make_row_keys(10);
        let style = default_style();
        let mut state = DataGridState {
            has_active_cell: true,
            active_row_key: 2000,
            active_column_key: 300,
            ..DataGridState::new()
        };

        set_grid_focus(&mut core, &layout, node);
        core.input.push_key(Key::Right);

        let _resp = data_grid_handle_keyboard(
            &mut core, &layout, node, &mut state, 10, &keys, &columns, &style,
        );

        assert_eq!(state.active_column_key, 200);
    }

    // -- Task 10: Performance regression test --

    #[test]
    fn draw_call_count_independent_of_total_rows() {
        let columns = make_columns(&[100.0, 150.0, 120.0, 80.0]);
        let style = default_style();

        let mut core_small = AkarCore::mock();
        core_small.draw_list.begin_frame(1.0);
        let (layout_s, node_s) = make_grid_layout(400.0, 300.0);
        let mut state_s = DataGridState::new();
        let keys_s = make_row_keys(1_000);
        let resp_s = data_grid_begin(
            &mut core_small,
            &layout_s,
            node_s,
            &mut state_s,
            1_000,
            &keys_s,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_header_begin(&mut core_small, &resp_s, &style);
        for ci in resp_s.visible_columns.clone() {
            if ci < columns.len() {
                data_grid_header_cell(
                    &mut core_small,
                    &layout_s,
                    &resp_s,
                    node_s,
                    ci,
                    &columns,
                    &style,
                    "H",
                    DataGridSortDirection::None,
                );
            }
        }
        data_grid_header_end(&mut core_small);
        data_grid_body_begin(&mut core_small, &resp_s, &keys_s, &style, &[]);
        for ri in resp_s.visible_rows.clone() {
            for ci in resp_s.visible_columns.clone() {
                if ci < columns.len() && ri < keys_s.len() {
                    data_grid_cell(
                        &mut core_small,
                        &layout_s,
                        &resp_s,
                        node_s,
                        ri,
                        keys_s[ri],
                        ci,
                        &columns,
                        &style,
                        "c",
                        false,
                    );
                }
            }
        }
        data_grid_body_end(&mut core_small);
        data_grid_end(&mut core_small);
        let count_small = core_small.draw_list.len();

        let mut core_large = AkarCore::mock();
        core_large.draw_list.begin_frame(1.0);
        let (layout_l, node_l) = make_grid_layout(400.0, 300.0);
        let mut state_l = DataGridState::new();
        let keys_l = make_row_keys(100_000);
        let resp_l = data_grid_begin(
            &mut core_large,
            &layout_l,
            node_l,
            &mut state_l,
            100_000,
            &keys_l,
            32.0,
            36.0,
            &columns,
            &style,
        );
        data_grid_header_begin(&mut core_large, &resp_l, &style);
        for ci in resp_l.visible_columns.clone() {
            if ci < columns.len() {
                data_grid_header_cell(
                    &mut core_large,
                    &layout_l,
                    &resp_l,
                    node_l,
                    ci,
                    &columns,
                    &style,
                    "H",
                    DataGridSortDirection::None,
                );
            }
        }
        data_grid_header_end(&mut core_large);
        data_grid_body_begin(&mut core_large, &resp_l, &keys_l, &style, &[]);
        for ri in resp_l.visible_rows.clone() {
            for ci in resp_l.visible_columns.clone() {
                if ci < columns.len() && ri < keys_l.len() {
                    data_grid_cell(
                        &mut core_large,
                        &layout_l,
                        &resp_l,
                        node_l,
                        ri,
                        keys_l[ri],
                        ci,
                        &columns,
                        &style,
                        "c",
                        false,
                    );
                }
            }
        }
        data_grid_body_end(&mut core_large);
        data_grid_end(&mut core_large);
        let count_large = core_large.draw_list.len();

        assert_eq!(count_small, count_large);
    }
}
