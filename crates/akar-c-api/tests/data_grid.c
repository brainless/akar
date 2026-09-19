#include "akar.h"

#include <assert.h>
#include <string.h>

static uint64_t make_sized_node(AkarCtx *ctx, float w, float h) {
    uint64_t root = akar_new_flex_col(ctx);
    uint64_t node = akar_new_fixed_leaf(ctx, w, h);
    akar_add_child(ctx, root, node);
    akar_layout_compute(ctx, root, 800.0f, 600.0f);
    return node;
}

void akar_run_data_grid_c_tests(void) {
    AkarCtx *ctx = akar_ctx_mock();
    assert(ctx != NULL);
    akar_begin_frame(ctx, 800, 600, 1.0f);

    AkarDataGridStyle style;
    akar_data_grid_style_default(ctx, &style);
    assert(style.row_height > 0.0f);
    assert(style.header_height > 0.0f);
    assert(style.font_size > 0.0f);

    {
        AkarDataGridState state = {0};
        uint64_t node = make_sized_node(ctx, 200.0f, 150.0f);
        AkarDataGridResponse resp = akar_data_grid_begin(
            ctx, node, &state, 0, NULL, 0, 32.0f, 36.0f, NULL, 0, &style
        );
        assert(resp.visible_row_start == 0);
        assert(resp.visible_row_end == 0);
        assert(resp.visible_column_start == 0);
        assert(resp.visible_column_end == 0);
        assert(resp.total_content_width == 0.0f);
        assert(resp.total_content_height == 0.0f);
        akar_data_grid_end(ctx);
    }

    {
        uint64_t node = make_sized_node(ctx, 400.0f, 300.0f);
        AkarDataGridColumn cols[2] = {
            {.key = 100, .width = 100.0f, .align = 0},
            {.key = 200, .width = 200.0f, .align = 0},
        };
        uint64_t row_keys[3] = {1000, 2000, 3000};
        AkarDataGridState state = {0};

        AkarDataGridResponse resp = akar_data_grid_begin(
            ctx, node, &state, 3, row_keys, 3, 32.0f, 36.0f, cols, 2, &style
        );
        assert(resp.visible_row_start == 0);
        assert(resp.visible_row_end > 0);
        assert(resp.visible_column_start == 0);
        assert(resp.visible_column_end == 2);
        assert(resp.total_content_width == 300.0f);
        assert(resp.total_content_height == 96.0f);
        assert(resp.has_active_cell == 0);
        assert(!resp.has_activated);
        assert(!resp.has_header_clicked);

        akar_data_grid_header_begin(ctx, &resp, cols, 2, &style);
        AkarDataGridHeaderResponse hdr0 = akar_data_grid_header_cell(
            ctx, node, &resp, 0, cols, 2, "Name", 0
        );
        assert(hdr0.column_key == 100);
        AkarDataGridHeaderResponse hdr1 = akar_data_grid_header_cell(
            ctx, node, &resp, 1, cols, 2, "Value", 0
        );
        assert(hdr1.column_key == 200);
        akar_data_grid_header_end(ctx);

        akar_data_grid_body_begin(ctx, &resp, row_keys, 3, cols, 2, &style, NULL, 0);
        AkarDataGridCellResponse cell0 = akar_data_grid_cell(
            ctx, node, &resp, 0, 1000, 0, cols, 2, "Alice", 0
        );
        assert(cell0.row_key == 1000);
        assert(cell0.column_key == 100);
        assert(cell0.row_index == 0);
        assert(cell0.column_index == 0);
        AkarDataGridCellResponse cell1 = akar_data_grid_cell(
            ctx, node, &resp, 1, 2000, 1, cols, 2, "Bob", 0
        );
        assert(cell1.row_key == 2000);
        assert(cell1.column_key == 200);
        akar_data_grid_body_end(ctx);

        akar_data_grid_end(ctx);
    }

    {
        uint64_t node = make_sized_node(ctx, 400.0f, 300.0f);
        AkarDataGridState state = {.scroll_y = 99999.0f};
        AkarDataGridColumn cols[1] = {
            {.key = 1, .width = 100.0f, .align = 0},
        };
        uint64_t keys[10] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
        AkarDataGridResponse resp = akar_data_grid_begin(
            ctx, node, &state, 10, keys, 10, 32.0f, 36.0f, cols, 1, &style
        );
        (void)resp;
        float body_h = 300.0f - 36.0f;
        float expected_max = 10.0f * 32.0f - body_h;
        if (expected_max < 0.0f) expected_max = 0.0f;
        assert(state.scroll_y <= expected_max + 0.5f);
        akar_data_grid_end(ctx);
    }

    {
        uint64_t node = make_sized_node(ctx, 400.0f, 300.0f);
        AkarDataGridColumn cols[2] = {
            {.key = 100, .width = 100.0f, .align = 0},
            {.key = 200, .width = 200.0f, .align = 0},
        };
        uint64_t keys[1] = {1000};
        AkarDataGridState state = {0};

        AkarDataGridResponse resp = akar_data_grid_begin(
            ctx, node, &state, 1, keys, 1, 32.0f, 36.0f, cols, 2, &style
        );
        akar_data_grid_header_begin(ctx, &resp, cols, 2, &style);
        akar_data_grid_header_end(ctx);
        akar_data_grid_body_begin(ctx, &resp, keys, 1, cols, 2, &style, NULL, 0);
        akar_data_grid_body_end(ctx);
        akar_data_grid_end(ctx);
    }

    {
        uint64_t node = make_sized_node(ctx, 400.0f, 300.0f);
        AkarDataGridColumn cols[1] = {
            {.key = 100, .width = 100.0f, .align = 0},
        };
        uint64_t keys[3] = {1000, 2000, 3000};
        AkarDataGridState state = {
            .active_row_key = 2000,
            .active_column_key = 100,
            .has_active_cell = 1,
        };

        AkarDataGridResponse resp = akar_data_grid_begin(
            ctx, node, &state, 3, keys, 3, 32.0f, 36.0f, cols, 1, &style
        );
        assert(resp.has_active_cell);
        assert(resp.active_row_key == 2000);

        AkarDataGridKeyboardResponse kb = akar_data_grid_handle_keyboard(
            ctx, node, &state, 3, keys, 3, cols, 1, &style
        );
        (void)kb;

        akar_data_grid_end(ctx);
    }

    akar_ctx_free(ctx);
}
