#ifndef AKAR_H
#define AKAR_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

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

typedef struct AkarRange {
    uint32_t start;
    uint32_t end;
} AkarRange;

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

#endif  /* AKAR_H */
