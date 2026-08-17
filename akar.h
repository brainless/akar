#ifndef AKAR_H
#define AKAR_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Unset value for `AkarTextStyle::font_family_name_handle`. Distinct from
 * `SENTINEL_U32` (0xFF), which would collide with a real font handle.
 */
#define AKAR_FONT_FAMILY_NAME_HANDLE_NONE UINT32_MAX

#define AKAR_SHORTCUT_MODIFIER_PRIMARY (1 << 0)

#define AKAR_SHORTCUT_MODIFIER_CONTROL (1 << 1)

#define AKAR_SHORTCUT_MODIFIER_SUPER (1 << 2)

#define AKAR_SHORTCUT_MODIFIER_ALT (1 << 3)

#define AKAR_SHORTCUT_MODIFIER_SHIFT (1 << 4)

#define AKAR_KEY_CHARACTER 11

#define AKAR_FONT_LOAD_OK 0

/**
 * Null context, null byte pointer, or zero length.
 */
#define AKAR_FONT_LOAD_INVALID_ARGUMENT 1

/**
 * The bytes contain no parsable font face.
 */
#define AKAR_FONT_LOAD_INVALID_DATA 2

/**
 * The bytes parsed but carry no font family.
 */
#define AKAR_FONT_LOAD_EMPTY_SOURCE 3

/**
 * A collection spanning more than one family; v1 accepts exactly one.
 */
#define AKAR_FONT_LOAD_MULTIPLE_FAMILIES 4

#define AKAR_KEY_BACKSPACE 0

#define AKAR_KEY_DELETE 1

#define AKAR_KEY_LEFT 2

#define AKAR_KEY_RIGHT 3

#define AKAR_KEY_UP 4

#define AKAR_KEY_DOWN 5

#define AKAR_KEY_HOME 6

#define AKAR_KEY_END 7

#define AKAR_KEY_ENTER 8

#define AKAR_KEY_ESCAPE 9

#define AKAR_KEY_TAB 10

typedef struct AkarCtx AkarCtx;

/**
 * Selects how the context populates its font database. Numeric tag carried as
 * a plain `uint32_t`, matching the existing `AkarFontFamily`/`AkarFontWeight`
 * convention.
 */
typedef uint32_t AkarFontSource;

typedef struct AkarRect {
    float x;
    float y;
    float w;
    float h;
} AkarRect;

typedef struct AkarButtonResult {
    bool clicked;
    bool hovered;
    bool pressed;
} AkarButtonResult;

typedef struct AkarBoxStyle {
    uint32_t fill;
    uint32_t border_color;
    float border_width;
    float corner_radii[4];
    uint32_t shadow_color;
    float shadow_offset[2];
    float shadow_blur;
    float shadow_spread;
} AkarBoxStyle;

typedef struct AkarDrawerResponse {
    bool close_requested;
} AkarDrawerResponse;

typedef struct AkarRange {
    uint32_t start;
    uint32_t end;
} AkarRange;

typedef struct AkarAlertResult {
    bool dismissed;
} AkarAlertResult;

typedef struct AkarNavbarSlots {
    uint64_t start;
    uint64_t center;
    uint64_t end;
} AkarNavbarSlots;

typedef struct AkarTabBarResponse {
    int32_t clicked_index;
} AkarTabBarResponse;

typedef struct AkarTooltipResponse {
    bool visible;
} AkarTooltipResponse;

typedef struct AkarModalResponse {
    bool close_requested;
    uint64_t content_node;
} AkarModalResponse;

typedef struct AkarToastResponse {
    int32_t dismissed;
} AkarToastResponse;

typedef struct AkarToastItem {
    uint32_t variant;
    const char *message;
    bool dismiss_on_click;
} AkarToastItem;

typedef struct AkarDropdownState {
    bool is_open;
    float content_rect[4];
} AkarDropdownState;

typedef struct AkarShortcut {
    uint32_t modifiers;
    uint32_t key;
    uint32_t character;
} AkarShortcut;

typedef struct AkarTextEditKeybindings {
    struct AkarShortcut select_all;
    struct AkarShortcut copy;
    struct AkarShortcut paste;
} AkarTextEditKeybindings;

typedef struct AkarSelectResponse {
    bool changed;
} AkarSelectResponse;

typedef struct AkarTextEditState {
    uint32_t cursor;
    uint32_t anchor;
} AkarTextEditState;

typedef struct AkarTextInputResponse {
    bool changed;
    bool submitted;
    uint64_t widget_id;
    struct AkarTextEditState edit_state;
    uint32_t copy_len;
    uint32_t copy_required_len;
    bool request_paste;
} AkarTextInputResponse;

typedef struct AkarDataItemStyle {
    float surface[4];
    float padding_x;
    float padding_y;
    float spacing;
    float color_normal[4];
    float color_hover[4];
    float color_pressed[4];
    float color_selected[4];
    float corner_radius;
    float border_width;
    float border_color[4];
} AkarDataItemStyle;

typedef struct AkarDataItemResponse {
    bool hovered;
    bool pressed;
    bool clicked;
} AkarDataItemResponse;

typedef struct AkarDataListResponse {
    float viewport_rect[4];
    float content_origin[2];
    uint32_t visible_range_start;
    uint32_t visible_range_end;
} AkarDataListResponse;

typedef struct AkarDataListState {
    float scroll_y;
} AkarDataListState;

typedef struct AkarTextAreaResponse {
    bool changed;
    uint64_t widget_id;
    struct AkarTextEditState edit_state;
    uint32_t copy_len;
    uint32_t copy_required_len;
    bool request_paste;
} AkarTextAreaResponse;

typedef struct AkarTextStyle {
    float font_size;
    float line_height;
    uint32_t color;
    uint32_t font_weight;
    uint32_t font_family;
    /**
     * Handle returned by `akar_load_font_bytes`, selecting a runtime-loaded
     * family and overriding `font_family`. Set to
     * `AKAR_FONT_FAMILY_NAME_HANDLE_NONE` when unused.
     */
    uint32_t font_family_name_handle;
    uint32_t align;
    uint8_t wrap;
} AkarTextStyle;

typedef struct AkarLinkResult {
    bool clicked;
    bool hovered;
    bool pressed;
} AkarLinkResult;

typedef struct AkarCardSlots {
    uint64_t header;
    uint64_t body;
    uint64_t footer;
} AkarCardSlots;

typedef struct AkarCardLayout {
    uint32_t direction;
    float gap;
    float padding;
    uint8_t has_header;
    uint8_t has_footer;
} AkarCardLayout;

typedef struct AkarCardStyle {
    uint32_t background;
    uint32_t border_color;
    float border_width;
    float corner_radii[4];
    float shadow_blur;
    float shadow_spread;
    uint32_t shadow_color;
    float shadow_offset[2];
    uint32_t separator_color;
} AkarCardStyle;

typedef struct AkarNavbarStyle {
    uint32_t background;
    uint32_t border_color;
    float border_width;
    float corner_radii[4];
} AkarNavbarStyle;

typedef struct AkarButtonStyle {
    uint32_t fill;
    uint32_t hover_fill;
    uint32_t pressed_fill;
    uint32_t border_color;
    uint32_t content_color;
    struct AkarTextStyle text_style;
} AkarButtonStyle;

typedef struct AkarBadgeStyle {
    uint32_t fill;
    uint32_t border_color;
    uint32_t content_color;
    struct AkarTextStyle text_style;
} AkarBadgeStyle;

typedef struct AkarSeparatorStyle {
    uint32_t color;
    float thickness;
} AkarSeparatorStyle;

typedef struct AkarStatStyle {
    uint32_t title_color;
    uint32_t value_color;
    uint32_t description_color;
    struct AkarTextStyle title_text_style;
    struct AkarTextStyle value_text_style;
    struct AkarTextStyle description_text_style;
} AkarStatStyle;

typedef struct AkarTabBarStyle {
    uint32_t active_color;
    uint32_t inactive_color;
    uint32_t indicator_color;
} AkarTabBarStyle;

typedef struct AkarFontFamily {
    uint32_t value;
} AkarFontFamily;

typedef struct AkarFontWeight {
    uint32_t value;
} AkarFontWeight;

typedef struct AkarTextAlign {
    uint32_t value;
} AkarTextAlign;

/**
 * Numeric tag for `akar_set_direction`, matching the existing
 * `AkarFontFamily`/`AkarFontWeight`/`AkarTextAlign` convention.
 */
typedef struct AkarDirection {
    uint32_t value;
} AkarDirection;

typedef struct AkarHeadingLevel {
    uint32_t value;
} AkarHeadingLevel;

/**
 * Bundled fonts only; no system font scanning. Deterministic across machines
 * and the default for every akar context.
 */
#define AKAR_FONT_SOURCE_BUNDLED 0

/**
 * Bundled fonts plus a full system font scan. Broader glyph coverage, but the
 * resolved faces are machine-dependent, so rendering is no longer reproducible
 * across machines or operating systems. Opt-in only.
 */
#define AKAR_FONT_SOURCE_BUNDLED_PLUS_SYSTEM_SCAN 1

/**
 * Creates a context bound to an existing wgpu device and queue.
 *
 * `font_source` is one of the `AKAR_FONT_SOURCE_*` constants; any unrecognized
 * value is treated as `AKAR_FONT_SOURCE_BUNDLED`.
 */
struct AkarCtx *akar_ctx_new(const void *device,
                             const void *queue,
                             uint32_t surface_format_raw,
                             AkarFontSource font_source);

void akar_ctx_free(struct AkarCtx *ctx);

/**
 * Creates a headless context suitable for testing layout and input logic.
 * The GPU pipeline is initialized against a headless wgpu adapter; no surface
 * or real window is required. Do not call `akar_end_frame` on a mock context.
 *
 * Takes no font source: a mock context is always pinned to
 * `AKAR_FONT_SOURCE_BUNDLED` so test rendering stays reproducible.
 */
struct AkarCtx *akar_ctx_mock(void);

/**
 * Loads font bytes (TTF/OTF/TTC/OTC) into the context's font database.
 *
 * Returns `AKAR_FONT_LOAD_OK` and writes the family handle to `out_handle`
 * (when non-NULL) on success, or one of the `AKAR_FONT_LOAD_*` error codes.
 * `out_handle` is left untouched on failure. Loading the same family twice
 * returns the same handle. Safe to call any time after context creation.
 */
uint32_t akar_load_font_bytes(struct AkarCtx *ctx,
                              const uint8_t *bytes,
                              uint32_t len,
                              uint32_t *out_handle);

void akar_begin_frame(struct AkarCtx *ctx, uint32_t width, uint32_t height, float scale_factor);

void akar_end_frame(struct AkarCtx *ctx, void *pass);

void akar_input_begin(struct AkarCtx *ctx);

void akar_set_mouse_pos(struct AkarCtx *ctx, float x, float y);

void akar_push_mouse_button(struct AkarCtx *ctx, uint32_t button, bool pressed);

void akar_push_scroll(struct AkarCtx *ctx, float dx, float dy);

void akar_push_char(struct AkarCtx *ctx, uint32_t codepoint);

void akar_input_end(struct AkarCtx *_ctx);

uint64_t akar_new_leaf(struct AkarCtx *ctx, float flex_grow);

uint64_t akar_new_fixed_leaf(struct AkarCtx *ctx, float w, float h);

uint64_t akar_new_flex_row(struct AkarCtx *ctx);

uint64_t akar_new_flex_col(struct AkarCtx *ctx);

void akar_add_child(struct AkarCtx *ctx, uint64_t parent, uint64_t child);

void akar_layout_compute(struct AkarCtx *ctx, uint64_t root, float width, float height);

struct AkarRect akar_layout_rect(struct AkarCtx *ctx, uint64_t node);

/**
 * Sets the layout direction (LTR/RTL) applied to every node created from
 * this point forward. Direction is stamped at node-creation time and does
 * not retroactively affect nodes already created; call this before building
 * the tree it should apply to. Returns `false` on an unrecognized
 * `direction` value or a null `ctx`.
 */
bool akar_set_direction(struct AkarCtx *ctx, uint32_t direction);

struct AkarButtonResult akar_button(struct AkarCtx *ctx,
                                    uint64_t node_id,
                                    const char *label,
                                    int32_t label_len);

void akar_label(struct AkarCtx *ctx,
                uint64_t node_id,
                const char *text,
                int32_t text_len,
                uint32_t color);

void akar_container(struct AkarCtx *ctx, uint64_t node_id, struct AkarBoxStyle style);

struct AkarDrawerResponse akar_drawer_begin(struct AkarCtx *ctx,
                                            uint32_t edge,
                                            float panel_width,
                                            const float *viewport_rect);

void akar_drawer_end(struct AkarCtx *ctx);

void akar_set_padding(struct AkarCtx *ctx,
                      uint64_t node_id,
                      float top,
                      float right,
                      float bottom,
                      float left);

void akar_set_margin(struct AkarCtx *ctx,
                     uint64_t node_id,
                     float top,
                     float right,
                     float bottom,
                     float left);

struct AkarRange akar_list_clip(uint32_t total,
                                float item_height,
                                float scroll_y,
                                float viewport_height);

float akar_scroll_area_begin(struct AkarCtx *ctx,
                             const float *rect,
                             float *scroll_y,
                             float content_height);

void akar_scroll_area_end(struct AkarCtx *ctx);

void akar_progress(struct AkarCtx *ctx,
                   uint64_t node_id,
                   float value,
                   uint32_t track_color,
                   uint32_t fill_color,
                   float corner_radius);

void akar_badge(struct AkarCtx *ctx, uint64_t node_id, const char *text, uint32_t variant);

struct AkarAlertResult akar_alert(struct AkarCtx *ctx,
                                  uint64_t node_id,
                                  const char *text,
                                  int32_t text_len,
                                  uint32_t variant,
                                  bool closable);

void akar_stat(struct AkarCtx *ctx,
               uint64_t node_id,
               const char *title,
               int32_t title_len,
               const char *value,
               int32_t value_len,
               const char *description,
               int32_t description_len);

void akar_skeleton(struct AkarCtx *ctx, uint64_t node_id, uint32_t variant);

struct AkarNavbarSlots akar_navbar(struct AkarCtx *ctx, uint64_t node_id);

void akar_steps(struct AkarCtx *ctx,
                uint64_t node_id,
                const char *const *labels,
                uint32_t label_count,
                const int32_t *label_lengths,
                uint32_t current);

struct AkarTabBarResponse akar_tab_bar(struct AkarCtx *ctx,
                                       uint64_t node_id,
                                       const char *const *labels,
                                       uint32_t label_count,
                                       const int32_t *label_lengths,
                                       uint32_t active_index,
                                       uint32_t variant);

void akar_avatar(struct AkarCtx *ctx,
                 uint64_t node_id,
                 const char *initials,
                 int32_t initials_len,
                 uint32_t color);

struct AkarTooltipResponse akar_tooltip(struct AkarCtx *ctx,
                                        const float *trigger_rect,
                                        const char *text,
                                        uint32_t preferred_side,
                                        const float *viewport_rect);

struct AkarModalResponse akar_modal_begin(struct AkarCtx *ctx,
                                          const char *title,
                                          int32_t title_len,
                                          float width,
                                          float height,
                                          const float *viewport_rect);

void akar_modal_end(struct AkarCtx *ctx);

struct AkarToastResponse akar_toasts(struct AkarCtx *ctx,
                                     const struct AkarToastItem *items,
                                     uint32_t item_count,
                                     const float *viewport_rect);

struct AkarDropdownState akar_dropdown_begin(struct AkarCtx *ctx,
                                             const float *anchor_rect,
                                             float item_height,
                                             const float *viewport_rect,
                                             bool is_open);

void akar_dropdown_end(struct AkarCtx *ctx);

struct AkarTextEditKeybindings akar_text_edit_keybindings_default(void);

bool akar_set_text_edit_keybindings(struct AkarCtx *ctx, struct AkarTextEditKeybindings bindings);

void akar_push_key(struct AkarCtx *ctx, uint32_t key);

void akar_push_key_event(struct AkarCtx *ctx,
                         uint32_t key,
                         uint32_t character,
                         uint32_t modifiers,
                         bool repeat);

bool akar_push_paste(struct AkarCtx *ctx,
                     uint64_t widget_id,
                     const uint8_t *utf8,
                     uint32_t utf8_len);

bool akar_checkbox(struct AkarCtx *ctx,
                   uint64_t node_id,
                   const char *label,
                   int32_t label_len,
                   bool *checked);

bool akar_radio_group(struct AkarCtx *ctx,
                      const uint64_t *nodes,
                      uint32_t node_count,
                      const char *const *labels,
                      const int32_t *label_lengths,
                      uint32_t *selected);

bool akar_switch(struct AkarCtx *ctx, uint64_t node_id, bool *on);

bool akar_slider(struct AkarCtx *ctx, uint64_t node_id, float *value, float min, float max);

struct AkarSelectResponse akar_select(struct AkarCtx *ctx,
                                      uint64_t node_id,
                                      const char *const *options,
                                      uint32_t option_count,
                                      const int32_t *option_lengths,
                                      uint32_t *selected,
                                      bool *open,
                                      const float *viewport_rect);

/**
 * Edits a caller-owned UTF-8 buffer.
 *
 * `value_len` is the meaningful byte length on input and receives the new
 * meaningful byte length. `value_capacity` is the allocation size in bytes.
 * Output is truncated only at a UTF-8 boundary and is NUL-terminated when the
 * resulting length is smaller than the capacity. Copy text is written to
 * `copy_buf`; `copy_len` reports bytes written and `copy_required_len` reports
 * the complete selected byte length.
 */
struct AkarTextInputResponse akar_text_input(struct AkarCtx *ctx,
                                             uint64_t node_id,
                                             uint8_t *value_buf,
                                             uint32_t *value_len,
                                             uint32_t value_capacity,
                                             struct AkarTextEditState *edit_state,
                                             const char *placeholder,
                                             bool cursor_visible,
                                             uint8_t *copy_buf,
                                             uint32_t copy_capacity);

void akar_data_item_style_default(struct AkarCtx *ctx, struct AkarDataItemStyle *style_out);

struct AkarDataItemResponse akar_data_item(struct AkarCtx *ctx,
                                           uint64_t node_id,
                                           uint64_t key,
                                           const struct AkarDataItemStyle *style);

struct AkarDataListResponse akar_data_list_begin(struct AkarCtx *ctx,
                                                 uint64_t node_id,
                                                 struct AkarDataListState *state,
                                                 uint32_t item_count,
                                                 float item_height,
                                                 const uint64_t *keys,
                                                 uint32_t key_count);

void akar_data_list_end(struct AkarCtx *ctx);

/**
 * Edits a caller-owned multiline UTF-8 buffer.
 *
 * Buffer and copy-output semantics match `akar_text_input`.
 */
struct AkarTextAreaResponse akar_textarea(struct AkarCtx *ctx,
                                          uint64_t node_id,
                                          uint8_t *value_buf,
                                          uint32_t *value_len,
                                          uint32_t value_capacity,
                                          struct AkarTextEditState *edit_state,
                                          float *scroll_y,
                                          const char *placeholder,
                                          bool cursor_visible,
                                          uint8_t *copy_buf,
                                          uint32_t copy_capacity);

void akar_heading(struct AkarCtx *ctx,
                  uint64_t node_id,
                  const char *text,
                  uint32_t level,
                  const struct AkarTextStyle *style);

void akar_paragraph(struct AkarCtx *ctx,
                    uint64_t node_id,
                    const char *text,
                    const struct AkarTextStyle *style);

struct AkarLinkResult akar_link(struct AkarCtx *ctx,
                                uint64_t node_id,
                                const char *text,
                                const struct AkarTextStyle *style);

struct AkarCardSlots akar_card_layout(struct AkarCtx *ctx,
                                      uint64_t node_id,
                                      const struct AkarCardLayout *options);

void akar_card(struct AkarCtx *ctx,
               uint64_t node_id,
               const struct AkarCardSlots *slots,
               const struct AkarCardStyle *style);

struct AkarNavbarSlots akar_navbar_layout(struct AkarCtx *ctx, uint64_t node_id);

void akar_navbar_painted(struct AkarCtx *ctx,
                         uint64_t node_id,
                         const struct AkarNavbarStyle *style);

struct AkarButtonResult akar_button_styled(struct AkarCtx *ctx,
                                           uint64_t node_id,
                                           const char *text,
                                           uint32_t variant,
                                           const struct AkarButtonStyle *style);

void akar_badge_styled(struct AkarCtx *ctx,
                       uint64_t node_id,
                       const char *text,
                       uint32_t variant,
                       const struct AkarBadgeStyle *style);

void akar_separator_styled(struct AkarCtx *ctx,
                           uint64_t node_id,
                           const struct AkarSeparatorStyle *style);

void akar_stat_styled(struct AkarCtx *ctx,
                      uint64_t node_id,
                      const char *title,
                      const char *value,
                      const char *description,
                      const struct AkarStatStyle *style);

struct AkarTabBarResponse akar_tab_bar_styled(struct AkarCtx *ctx,
                                              uint64_t node_id,
                                              const char *const *tabs,
                                              uint32_t tab_count,
                                              uint32_t active_tab,
                                              uint32_t variant,
                                              const struct AkarTabBarStyle *style);

#endif  /* AKAR_H */
