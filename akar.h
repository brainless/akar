#ifndef AKAR_H
#define AKAR_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct AkarCtx AkarCtx;

typedef struct AkarButtonResult {
    bool clicked;
    bool hovered;
    bool pressed;
} AkarButtonResult;

struct AkarCtx *akar_ctx_new(const void *device, const void *queue, uint32_t surface_format_raw);

void akar_ctx_free(struct AkarCtx *ctx);

void akar_begin_frame(struct AkarCtx *ctx, uint32_t width, uint32_t height, float scale_factor);

void akar_end_frame(struct AkarCtx *ctx, void *pass);

void akar_input_begin(struct AkarCtx *ctx);

void akar_set_mouse_pos(struct AkarCtx *ctx, float x, float y);

void akar_push_mouse_button(struct AkarCtx *ctx, uint32_t button, bool pressed);

void akar_push_scroll(struct AkarCtx *ctx, float dx, float dy);

void akar_push_char(struct AkarCtx *ctx, uint32_t codepoint);

void akar_input_end(struct AkarCtx *_ctx);

struct AkarButtonResult akar_button(struct AkarCtx *ctx,
                                    uint64_t node_id,
                                    const char *label,
                                    int32_t label_len);

#endif  /* AKAR_H */
