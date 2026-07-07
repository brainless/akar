# Epic 011: Overlay Stack

**Status:** In Progress
**Goal:** Floating UI elements — Tooltip, Modal/Dialog, Toast, and the Dropdown primitive. All render at `Z_OVERLAY` (introduced in Epic 010), above the drawer layer. This epic completes the layering model and provides the infrastructure that Select (Epic 012) depends on for its option list.

**Prerequisite:** Epic 010 is `Status: Done` and `cargo clippy --workspace -- -D warnings` passes clean.

---

## Task Notes

### Task 1 — Tooltip + position_tooltip (Done)

`crates/akar-components/src/tooltip.rs`:
- `TooltipSide` enum: `Top`, `Bottom`, `Left`, `Right`
- `position_tooltip()` — public free function; places rect relative to trigger, flips to opposite side when preferred side doesn't fit viewport, clamps result to viewport bounds
- `tooltip()` — renders only when trigger rect is hovered; measures text via `text_pipeline.set_text()` + `measure()`, positions via `position_tooltip()`, pushes rounded quad background + text at `Z_OVERLAY`
- 5 tests: not-hovered (0 calls, visible=false), hovered (≥1 call, visible=true), above placement, flip-to-bottom, right-edge clamping
- All 5 pass, 54 total component tests, clippy clean

### Task 2 — Modal/Dialog component (Done)

`crates/akar-components/src/modal.rs`:
- `ModalResponse { close_requested: bool, content_node: NodeId }`
- `modal_begin()` — scrim quad at `Z_SCRIM`, panel quad at `Z_FLOAT`, internal taffy sub-tree (title + "×" close button header, content area), scissor push to content rect; `close_requested` fires on scrim click or close button click
- `modal_end()` — pops scissor
- `content_node` is a valid taffy NodeId for caller rendering
- 5 tests: zero size (0 calls), renders ≥2 quads + ≥4 total calls, scrim click closes, valid content node, scissor balanced
- 59 total component tests, clippy clean

### Task 3 — Toast component (Done)

`crates/akar-components/src/toast.rs`:
- `ToastVariant` enum: `Info`, `Success`, `Warning`, `Error`
- `ToastItem { variant, message, dismiss_on_click }` and `ToastResponse { dismissed: Option<usize> }`
- `toasts()` — renders each item as rounded quad at `Z_OVERLAY` with variant-colored fill, white text; stacks from bottom-right upward; width = 35% viewport (max 360px); returns `Some(index)` on click if `dismiss_on_click` is true
- 6 tests: empty (0 calls), single (≥2 calls), 3 stack (3 quads, right-aligned, distinct Y), dismiss click, non-dismissable, all 4 variants (≥8 calls)
- 65 total component tests, clippy clean

---

## Scope

### Components

#### Tooltip
A small text label that appears near a trigger element when the trigger is hovered. The tooltip rect is positioned relative to the trigger rect (above, below, left, or right) and clipped to the viewport bounds to avoid off-screen rendering. No arrow/caret in v1.

The caller passes the trigger rect (from `layout.rect(node_id)`) and a text string. `tooltip` returns a `TooltipResponse { visible: bool }` and renders only when `visible` is true (i.e., the trigger is hovered). The caller decides what constitutes the trigger — any rect can be a tooltip anchor.

#### Modal / Dialog
A centered floating panel with a scrim backdrop. The modal renders at `Z_FLOAT` (same as drawer — one modal open at a time); its content is caller-rendered. The backdrop scrim covers the full viewport at `Z_SCRIM`.

`modal_begin(ctx, title, viewport_rect, width, height) -> ModalResponse`  
`modal_end(ctx)`  
`ModalResponse { close_requested: bool, content_node: NodeId }` — `close_requested` is true when the scrim is clicked or the default close button is activated. `content_node` is the taffy node the caller renders into.

The modal panel uses `BoxStyle::card` plus a header bar (title text + close button) and a content region. Layout is an independent sub-tree managed internally by the modal component.

#### Toast
A non-blocking notification that appears in a corner of the viewport (default: bottom-right). Multiple toasts stack vertically. The caller manages a `Vec<Toast>` and passes it each frame. Toasts have a `variant` (Info, Success, Warning, Error), a `message` string, and a `dismiss_on_click: bool` flag. Expired toasts are the caller's responsibility to remove (akar does not track time — the caller tracks elapsed frames or duration).

`toasts(ctx, viewport_rect, &mut Vec<ToastItem>) -> ToastResponse`  
`ToastResponse { dismissed: Option<usize> }` — index of the toast the user clicked (if `dismiss_on_click`).

#### Dropdown (primitive)
A positioned list of text options that opens below (or above, if near the viewport bottom) an anchor rect. Used internally by `Select` (Epic 012) and available for custom use.

`dropdown_begin(ctx, anchor_rect, item_height, viewport_rect) -> DropdownState`  
The caller renders each item inside the open dropdown using standard components (`label`, `button`). `dropdown_end(ctx)` closes the list.  
`DropdownState { is_open: bool, content_rect: [f32; 4] }`.

The dropdown renders a card-style background at `Z_OVERLAY`, scissor-clipped to its content rect. The caller controls open/closed state via their own `bool`.

---

## Key Design Decisions

### One Floating Panel at a Time (v1)

Modals and drawers share the `Z_FLOAT` layer. In v1, opening both simultaneously produces undefined z-ordering between them — this is acceptable. A full z-stack with ordered panel management is deferred beyond v1.

### Tooltip Positioning Logic

Tooltip position is computed as follows:
1. Try to place above the trigger. If the tooltip would go above the viewport top, place below.
2. Try to center horizontally on the trigger. If it would clip the right edge, shift left.
3. The result is clamped to the viewport rect.

This is a pure function: `position_tooltip(trigger_rect, tooltip_size, viewport_rect, preferred_side) -> [f32; 4]`. Exposed as a free function so callers can use it for custom floating elements.

### Toast Lifetime

akar does not own a timer. The caller passes a `duration_remaining: f32` field (frames or seconds — caller decides units). `toasts` renders all items in the slice and returns which one was clicked. The caller removes expired or clicked toasts from their Vec after the call. This keeps akar free of time-tracking state.

### Dropdown Positioning

The dropdown opens below the anchor rect by default. If the bottom of the dropdown would exceed the viewport bottom, it opens above the anchor instead. Width matches the anchor rect. This logic is consistent with browser `<select>` behavior.

---

## C ABI

- `akar_tooltip(ctx, trigger_rect, text, preferred_side)` — renders if trigger is hovered.
- `akar_modal_begin(ctx, title, width, height, viewport_rect) -> AkarModalResponse`
- `akar_modal_end(ctx)`
- `akar_toasts(ctx, items, item_count, viewport_rect) -> AkarToastResponse`
- `akar_dropdown_begin(ctx, anchor_rect, item_height, viewport_rect) -> AkarDropdownState`
- `akar_dropdown_end(ctx)`

`AkarToastItem` is a repr(C) struct: `{ variant: u32, message: *const c_char, dismiss_on_click: bool }`.

---

## Demo

The demo gains:
- A "More info" tooltip on the progress bars from Epic 008.
- A "New Item" button in the navbar that opens a modal with a title and placeholder content.
- Three auto-stacking toasts triggered by the tab-switch events from Epic 010.
- A simple dropdown button to demonstrate the `dropdown_begin/end` primitive.

---

## Acceptance Criteria

- [ ] `tooltip` renders only when the trigger rect is hovered; position is viewport-clamped.
- [ ] `position_tooltip` is a public free function usable independently of the tooltip component.
- [ ] `modal_begin/end` renders a centered panel with scrim; `close_requested` fires on scrim click.
- [ ] Modal content node is a valid taffy NodeId the caller can render into.
- [ ] `toasts` renders all items stacked in the correct corner; `dismissed` index is correct.
- [ ] `dropdown_begin/end` renders a card-background list at Z_OVERLAY; opens above anchor if near viewport bottom.
- [ ] All components exposed in `akar.h`.
- [ ] `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` pass clean.
- [ ] No retained timer state — toast lifetime is caller-managed.
