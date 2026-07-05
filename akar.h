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

#endif  /* AKAR_H */
