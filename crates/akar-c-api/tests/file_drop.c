#include "akar.h"

#include <assert.h>
#include <string.h>

static AkarFilePathInput path_input(const uint8_t *data, uint32_t len, uint32_t encoding) {
    AkarFilePathInput input = {data, len, encoding};
    return input;
}

void akar_run_file_drop_c_tests(void) {
    AkarCtx *ctx = akar_ctx_mock();
    assert(ctx != NULL);
    uint64_t root = akar_new_flex_col(ctx);
    uint64_t form = akar_new_fixed_leaf(ctx, 300.0f, 200.0f);
    akar_add_child(ctx, root, form);
    akar_layout_compute(ctx, root, 800.0f, 600.0f);
    akar_begin_frame(ctx, 800, 600, 1.0f);
    akar_input_begin(ctx);

    const uint8_t first[] = {'/', 't', 'm', 'p', '/', 'o', 'n', 'e'};
    const uint8_t second[] = {'/', 't', 'm', 'p', '/', 't', 'w', 'o'};
    AkarFilePathInput paths[] = {
        path_input(first, sizeof(first), AKAR_FILE_ENCODING_UTF8),
        path_input(second, sizeof(second), AKAR_FILE_ENCODING_UTF8),
    };
    assert(akar_file_drag_enter(ctx, 20.0f, 30.0f, paths, 2) == AKAR_FILE_OK);
    AkarFileTargetResponse response = akar_file_drop_target(ctx, form);
    assert(response.status == AKAR_FILE_OK && response.hovered && response.path_count == 0);
    assert(!akar_file_drop_target(ctx, form).hovered);
    assert(akar_file_drag_move(ctx, 30.0f, 40.0f) == AKAR_FILE_OK);
    assert(akar_file_drag_leave(ctx) == AKAR_FILE_OK);
    assert(!akar_file_drop_target(ctx, form).hovered);

    assert(akar_file_drop(ctx, 20.0f, 30.0f, paths, 2) == AKAR_FILE_OK);
    response = akar_file_drop_target(ctx, form);
    assert(response.status == AKAR_FILE_OK && response.path_count == 2);
    AkarFilePathInfo info = {0};
    assert(akar_file_target_path_info(ctx, 0, &info) == AKAR_FILE_OK);
#if defined(_WIN32)
    assert(info.encoding == AKAR_FILE_ENCODING_WINDOWS_UTF16);
    assert(info.required_bytes == sizeof(first) * sizeof(uint16_t));
#else
    assert(info.encoding == AKAR_FILE_ENCODING_UNIX_BYTES);
    assert(info.required_bytes == sizeof(first));
#endif
    uint8_t output[64] = {0};
    assert(akar_file_target_path_copy(ctx, 0, output, 1) == AKAR_FILE_BUFFER_TOO_SMALL);
    assert(output[0] == 0);
    assert(akar_file_target_path_copy(ctx, 0, NULL, info.required_bytes) == AKAR_FILE_INVALID_ARGUMENT);
    assert(akar_file_target_path_copy(ctx, 0, output, sizeof(output)) == AKAR_FILE_OK);
#if !defined(_WIN32)
    assert(memcmp(output, first, sizeof(first)) == 0);
#endif
    assert(akar_file_target_path_info(ctx, 1, &info) == AKAR_FILE_OK);
    assert(akar_file_target_path_copy(ctx, 1, output, sizeof(output)) == AKAR_FILE_OK);
#if !defined(_WIN32)
    assert(memcmp(output, second, sizeof(second)) == 0);
#endif
    assert(akar_file_target_path_info(ctx, 2, &info) == AKAR_FILE_INDEX_OUT_OF_RANGE);
    assert(akar_file_drop_target(ctx, form).path_count == 0);
    assert(akar_file_target_path_info(ctx, 0, &info) == AKAR_FILE_INDEX_OUT_OF_RANGE);

    assert(akar_file_drop(ctx, 20.0f, 30.0f, NULL, 1) == AKAR_FILE_INVALID_ARGUMENT);
    AkarFilePathInput bad = path_input(NULL, 1, AKAR_FILE_ENCODING_UTF8);
    assert(akar_file_drop(ctx, 20.0f, 30.0f, &bad, 1) == AKAR_FILE_INVALID_ARGUMENT);
    const uint8_t invalid_utf8[] = {0xff};
    bad = path_input(invalid_utf8, sizeof(invalid_utf8), AKAR_FILE_ENCODING_UTF8);
    assert(akar_file_drop(ctx, 20.0f, 30.0f, &bad, 1) == AKAR_FILE_INVALID_ENCODING);
    bad = path_input(first, sizeof(first), 99);
    assert(akar_file_drop(ctx, 20.0f, 30.0f, &bad, 1) == AKAR_FILE_INVALID_ENCODING);
    assert(akar_file_target_path_info(NULL, 0, &info) == AKAR_FILE_INVALID_ARGUMENT);
    assert(akar_file_target_path_info(ctx, 0, NULL) == AKAR_FILE_INVALID_ARGUMENT);

#if defined(_WIN32)
    const uint16_t native[] = {'C', ':', '\\', 0x00e9};
    AkarFilePathInput raw = path_input((const uint8_t *)native, sizeof(native), AKAR_FILE_ENCODING_WINDOWS_UTF16);
#else
    const uint8_t native[] = {'/', 't', 'm', 'p', '/', 0xff};
    AkarFilePathInput raw = path_input(native, sizeof(native), AKAR_FILE_ENCODING_UNIX_BYTES);
#endif
    assert(akar_file_drop(ctx, 20.0f, 30.0f, &raw, 1) == AKAR_FILE_OK);
    assert(akar_file_drop_target(ctx, form).path_count == 1);
    assert(akar_file_target_path_info(ctx, 0, &info) == AKAR_FILE_OK);
    assert(info.required_bytes == sizeof(native));
    assert(akar_file_target_path_copy(ctx, 0, output, sizeof(output)) == AKAR_FILE_OK);
    assert(memcmp(output, native, sizeof(native)) == 0);

    akar_input_begin(ctx);
    assert(akar_file_target_path_info(ctx, 0, &info) == AKAR_FILE_INDEX_OUT_OF_RANGE);
    assert(akar_file_drop_target(ctx, form).path_count == 0);
    assert(akar_file_drop_unpositioned(ctx, paths, 2) == AKAR_FILE_OK);
    assert(akar_file_drop_target(ctx, form).path_count == 0);
    akar_ctx_free(ctx);
}
