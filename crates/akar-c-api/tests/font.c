#include "akar.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static uint8_t *read_file(const char *path, uint32_t *out_len) {
    FILE *file = fopen(path, "rb");
    assert(file != NULL);
    assert(fseek(file, 0, SEEK_END) == 0);
    long size = ftell(file);
    assert(size > 0);
    assert(fseek(file, 0, SEEK_SET) == 0);

    uint8_t *bytes = (uint8_t *)malloc((size_t)size);
    assert(bytes != NULL);
    assert(fread(bytes, 1, (size_t)size, file) == (size_t)size);
    fclose(file);

    *out_len = (uint32_t)size;
    return bytes;
}

static AkarTextStyle unset_text_style(void) {
    return (AkarTextStyle){
        .font_size = 0.0f,
        .line_height = 0.0f,
        .color = 0,
        .font_weight = 0xFF,
        .font_family = 0xFF,
        .font_family_name_handle = AKAR_FONT_FAMILY_NAME_HANDLE_NONE,
        .align = 0xFF,
        .wrap = 0xFF,
    };
}

void akar_run_font_c_tests(void) {
    AkarCtx *ctx = akar_ctx_mock();
    assert(ctx != NULL);

    uint32_t garbage_handle = 0xABCDu;
    uint8_t garbage[64];
    memset(garbage, 0, sizeof(garbage));
    assert(
        akar_load_font_bytes(ctx, garbage, (uint32_t)sizeof(garbage), &garbage_handle)
        == AKAR_FONT_LOAD_INVALID_DATA
    );
    assert(garbage_handle == 0xABCDu);

    assert(
        akar_load_font_bytes(ctx, NULL, 4, &garbage_handle)
        == AKAR_FONT_LOAD_INVALID_ARGUMENT
    );
    assert(
        akar_load_font_bytes(ctx, garbage, 0, &garbage_handle)
        == AKAR_FONT_LOAD_INVALID_ARGUMENT
    );
    assert(
        akar_load_font_bytes(NULL, garbage, (uint32_t)sizeof(garbage), &garbage_handle)
        == AKAR_FONT_LOAD_INVALID_ARGUMENT
    );

    uint32_t font_len = 0;
    uint8_t *font_bytes = read_file(AKAR_TEST_FONT_PATH, &font_len);

    uint32_t handle = 0xFFFFFFFFu;
    assert(akar_load_font_bytes(ctx, font_bytes, font_len, &handle) == AKAR_FONT_LOAD_OK);
    assert(handle != AKAR_FONT_FAMILY_NAME_HANDLE_NONE);

    uint32_t handle_again = 0xFFFFFFFFu;
    assert(akar_load_font_bytes(ctx, font_bytes, font_len, &handle_again) == AKAR_FONT_LOAD_OK);
    assert(handle_again == handle);

    assert(akar_load_font_bytes(ctx, font_bytes, font_len, NULL) == AKAR_FONT_LOAD_OK);

    akar_begin_frame(ctx, 800, 600, 1.0f);
    uint64_t node = akar_new_fixed_leaf(ctx, 400.0f, 60.0f);
    uint64_t root = akar_new_flex_col(ctx);
    akar_add_child(ctx, root, node);
    akar_layout_compute(ctx, root, 800.0f, 600.0f);

    AkarTextStyle named = unset_text_style();
    named.font_family_name_handle = handle;
    akar_paragraph(ctx, node, "Named family round trip", &named);
    akar_heading(ctx, node, "Named heading", 0, &named);

    free(font_bytes);
    akar_ctx_free(ctx);
}
