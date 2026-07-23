#include "akar.h"

#include <assert.h>
#include <string.h>

static AkarShortcut alt_character(char character) {
    return (AkarShortcut){
        .modifiers = AKAR_SHORTCUT_MODIFIER_ALT,
        .key = AKAR_KEY_CHARACTER,
        .character = (uint32_t)character,
    };
}

static AkarTextInputResponse render_text_input(
    AkarCtx *ctx,
    uint64_t node,
    uint8_t *value,
    uint32_t *value_len,
    uint32_t value_capacity,
    AkarTextEditState *state,
    uint8_t *copy,
    uint32_t copy_capacity
) {
    return akar_text_input(
        ctx,
        node,
        value,
        value_len,
        value_capacity,
        state,
        "",
        true,
        copy,
        copy_capacity
    );
}

void akar_run_text_edit_c_tests(void) {
    AkarCtx *ctx = akar_ctx_mock();
    assert(ctx != NULL);
    akar_begin_frame(ctx, 800, 600, 1.0f);

    uint64_t node = akar_new_fixed_leaf(ctx, 240.0f, 40.0f);
    uint64_t root = akar_new_flex_col(ctx);
    akar_add_child(ctx, root, node);
    akar_layout_compute(ctx, root, 800.0f, 600.0f);

    AkarTextEditKeybindings bindings = {
        .select_all = alt_character('x'),
        .copy = alt_character('y'),
        .paste = alt_character('z'),
    };
    assert(akar_set_text_edit_keybindings(ctx, bindings));

    uint8_t value[5] = {'h', 'e', 'l', 'l', 'o'};
    uint32_t value_len = 5;
    AkarTextEditState state = {.cursor = 5, .anchor = 5};

    akar_input_begin(ctx);
    akar_set_mouse_pos(ctx, 10.0f, 10.0f);
    akar_push_mouse_button(ctx, 0, true);
    akar_push_mouse_button(ctx, 0, false);
    (void)render_text_input(ctx, node, value, &value_len, 5, &state, NULL, 0);

    akar_input_begin(ctx);
    akar_push_key_event(
        ctx,
        AKAR_KEY_CHARACTER,
        (uint32_t)'x',
        AKAR_SHORTCUT_MODIFIER_ALT,
        false
    );
    AkarTextInputResponse selected =
        render_text_input(ctx, node, value, &value_len, 5, &state, NULL, 0);
    assert(selected.edit_state.anchor == 0);
    assert(selected.edit_state.cursor == 5);

    uint8_t copy[8] = {0};
    akar_input_begin(ctx);
    akar_push_key_event(
        ctx,
        AKAR_KEY_CHARACTER,
        (uint32_t)'y',
        AKAR_SHORTCUT_MODIFIER_ALT,
        false
    );
    AkarTextInputResponse copied =
        render_text_input(ctx, node, value, &value_len, 5, &state, copy, 8);
    assert(copied.copy_len == 5);
    assert(copied.copy_required_len == 5);
    assert(memcmp(copy, "hello", 5) == 0);
    assert(copy[5] == 0);

    akar_input_begin(ctx);
    akar_push_key_event(
        ctx,
        AKAR_KEY_CHARACTER,
        (uint32_t)'z',
        AKAR_SHORTCUT_MODIFIER_ALT,
        false
    );
    AkarTextInputResponse paste_request =
        render_text_input(ctx, node, value, &value_len, 5, &state, NULL, 0);
    assert(paste_request.request_paste);
    assert(paste_request.widget_id == node);

    const uint8_t utf8_paste[] = {0xc3, 0xa9, 0xc3, 0xa9, 0xc3, 0xa9};
    akar_input_begin(ctx);
    assert(akar_push_paste(
        ctx,
        paste_request.widget_id,
        utf8_paste,
        (uint32_t)sizeof(utf8_paste)
    ));
    AkarTextInputResponse pasted =
        render_text_input(ctx, node, value, &value_len, 5, &state, NULL, 0);
    assert(pasted.changed);
    assert(value_len == 4);
    assert(memcmp(value, utf8_paste, 4) == 0);
    assert(value[4] == 0);
    assert(state.cursor == 4);
    assert(state.anchor == 4);

    memset(copy, 0x55, sizeof(copy));
    akar_input_begin(ctx);
    akar_push_key_event(
        ctx,
        AKAR_KEY_CHARACTER,
        (uint32_t)'y',
        AKAR_SHORTCUT_MODIFIER_ALT,
        false
    );
    AkarTextInputResponse empty_copy =
        render_text_input(ctx, node, value, &value_len, 5, &state, copy, 8);
    assert(empty_copy.copy_len == 0);
    assert(empty_copy.copy_required_len == 0);
    for (size_t index = 0; index < sizeof(copy); ++index) {
        assert(copy[index] == 0x55);
    }

    akar_ctx_free(ctx);
}
