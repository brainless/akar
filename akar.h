#ifndef AKAR_H
#define AKAR_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

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

typedef struct AkarSelectResponse {
    bool changed;
} AkarSelectResponse;

typedef struct AkarTextInputResponse {
    bool changed;
    bool submitted;
    uint32_t new_cursor_pos;
} AkarTextInputResponse;

typedef struct AkarTextAreaResponse {
    bool changed;
    uint32_t new_cursor_pos;
} AkarTextAreaResponse;

struct AkarCtx *akar_ctx_new(const void *device, const void *queue, uint32_t surface_format_raw);

void akar_ctx_free(struct AkarCtx *ctx);

/**
 * Creates a headless context suitable for testing layout and input logic.
 * The GPU pipeline is initialized against a headless wgpu adapter; no surface
 * or real window is required. Do not call `akar_end_frame` on a mock context.
 */
struct AkarCtx *akar_ctx_mock(void);

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

void akar_push_key(struct AkarCtx *ctx, uint32_t key);

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

struct AkarTextInputResponse akar_text_input(struct AkarCtx *ctx,
                                             uint64_t node_id,
                                             uint8_t *value_buf,
                                             uint32_t buf_len,
                                             uint32_t *cursor_pos,
                                             const char *placeholder,
                                             bool cursor_visible);

struct AkarTextAreaResponse akar_textarea(struct AkarCtx *ctx,
                                          uint64_t node_id,
                                          uint8_t *value_buf,
                                          uint32_t buf_len,
                                          uint32_t *cursor_pos,
                                          float *scroll_y,
                                          const char *placeholder,
                                          bool cursor_visible);

#endif  /* AKAR_H */
