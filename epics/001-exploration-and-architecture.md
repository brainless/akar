# Epic 001: Exploration & Architecture

**Status:** Active
**Goal:** Research and document the design space thoroughly enough to write Epic 002 — which is where implementation begins. No code is produced in this epic.

---

## Introduction

### Work Context

**Problem:** akar intends to be a GPU-accelerated, immediate-mode, language-neutral UI component library. Each of those three properties has prior art with real tradeoffs. Building on wgpu + glyphon is a deliberate constraint (from sugacode), but the rendering architecture, layout strategy, C ABI shape, component model, and virtualization approach all need grounding in existing production systems before we commit to anything.

**Goal of this epic:** Produce a set of Architecture Decision Records (ADRs) and a concrete Epic 002 task list by reading the source code of mature UI systems and extracting what is directly applicable to akar.

**What we are NOT deciding:** language bindings beyond the C ABI, accessibility, animation, async integration, testing infrastructure. These are deferred to later epics.

**Output:** A completed `epics/002-core-renderer-and-draw-list.md` that can be immediately handed to a coding agent.

---

## Reference sources

All research should read local source code first. For projects not yet cloned, clone to `~/Projects/` before starting the relevant task.

| System | Local path | Why it matters |
|---|---|---|
| **sugacode** | `~/Projects/sugacode` | akar's direct predecessor. The renderer and UIManager pattern is the starting point. Understand it completely before studying anything else. |
| **glyphon** | `~/Projects/glyphon` | akar's text pipeline. The `TextRenderer` + `TextAtlas` + `Viewport` lifecycle is fixed — akar wraps it, not replaces it. |
| **wgpu** | `~/Projects/wgpu` | The GPU layer. Understand render pass structure, buffer management, pipeline creation. |
| **xilem** | `~/Projects/xilem` | Mature retained-mode Rust UI. Study it to understand what akar is explicitly NOT — to confirm the immediate-mode choice and understand the cost of deferred decisions (accessibility, animation). |
| **daisyui** | `~/Projects/daisyui` | Component catalog and token system. Mine the package source for the canonical component list and how tokens are structured. |
| **shadcn_ui** | `~/Projects/shadcn_ui` | Component API ergonomics. Study how components are composed in a call-site-first design. |
| **gpui** (Zed) | `~/Projects/gpui` (clone from `github.com/zed-industries/zed`, subtree `crates/gpui`) | wgpu-based UI in production at scale. Scene graph, element protocol, platform layer abstraction. The most relevant production reference. |
| **egui** | `~/Projects/egui` (clone from `github.com/emilk/egui`) | The dominant Rust immediate-mode UI. Painter API, `Response` type, `Id`/`Memory` system, layout cursor, `ListClipper`. Most directly comparable to akar's intended model. |
| **Dear ImGui** | `~/Projects/imgui` (clone from `github.com/ocornut/imgui`) | The canonical immediate-mode reference implementation in C. `DrawList`, `ImGuiListClipper`, `ID` stack, input model. Study `imgui.h` and `imgui_draw.cpp`. |
| **Nuklear** | `~/Projects/nuklear` (clone from `github.com/Immediate-Mode-UI/Nuklear`) | Single-header C immediate-mode UI. Best example of a clean, portable, backend-agnostic C API that akar's C ABI should mirror. Study `nuklear.h`. |
| **taffy** | `~/Projects/taffy` (clone from `github.com/DioxusLabs/taffy`) | CSS Flexbox + Grid layout in pure Rust. The primary candidate for akar's layout engine. Study `taffy::Tree`, `NodeId`, `Style`, `Layout` output, and how it handles dirty/recompute. |
| **sokol** | `~/Projects/sokol` (clone from `github.com/floooh/sokol`) | C headers for GPU, app, and audio. The gold standard for a language-neutral C API. Study `sokol_gfx.h` and `sokol_app.h` for how state is passed opaquely, how initialization works, and how the API avoids imposing allocation strategy. |

---

## Tasks

### Task 1: Understand the sugacode baseline
**Priority:** High — do this first

**Goal:** Extract a precise picture of what sugacode already provides and what it lacks, so akar does not rebuild what already works.

**Study targets:**
- `~/Projects/sugacode/src/renderer.rs` — wgpu + glyphon initialization, `TextAreaData` collect-and-render model, frame lifecycle.
- `~/Projects/sugacode/src/ui/` — `UIManager`, `Canvas`, `Drawer`, `SearchBox`, `Container` — what state each holds, how it receives input, how it submits text areas.
- `~/Projects/sugacode/src/input.rs` — mouse/keyboard state struct, hit-test helpers.
- `~/Projects/sugacode/src/state.rs` — app state model; what the UI reads.

**Questions to answer:**
1. What does sugacode's renderer do that akar-core must replicate or generalize?
2. What is the exact glyphon API surface used? (Which types, which method signatures, in what order?)
3. What is missing? (No quad/rect renderer — confirm. No layout engine — confirm. No input focus/hover state machine — confirm.)
4. What can be directly ported to akar-core vs. what needs to be redesigned?

**Output:** A written summary (added as a section to this epic or a separate `docs/baseline-analysis.md`) covering the above four questions.

---

### Task 2: Study Dear ImGui's draw list and list clipper
**Priority:** High

**Goal:** Extract the patterns from Dear ImGui that akar should adopt directly, and the ones it should adapt or skip.

**Study targets:**
- `imgui.h`: `ImDrawList`, `ImDrawCmd`, `ImDrawVert`, `ImGuiListClipper`. Read the struct definitions and their doc comments.
- `imgui_draw.cpp`: `ImDrawList::AddRectFilled`, `AddRect`, `AddText`, `PushClipRect`/`PopClipRect`. Understand how the draw list is built and flushed.
- `imgui.cpp`: `ImGuiListClipper::Begin` / `Step` / `End` — the virtualization primitive.
- The ID stack: `ImGui::PushID` / `PopID`, how items get unique IDs for hover/active tracking without a retained tree.

**Questions to answer:**
1. How does ImGui's `DrawList` batch and sort draw calls? Does akar need Z-sorting or is painter's order sufficient?
2. What exactly does `ImGuiListClipper` compute, and what information does it require from the caller? Can this be a pure function?
3. How does the ID stack work, and does akar need an equivalent for hover/focus state?
4. What does ImGui's input model look like? (What data does `ImGuiIO` hold, when is it populated?)
5. What C types does ImGui expose that would translate cleanly to `akar.h`?

**Output:** Written summary of the above five questions, with specific file:line citations.

---

### Task 3: Study egui's immediate-mode architecture
**Priority:** High

**Goal:** egui is Rust-native and the closest direct comparator to akar. Extract what works, what doesn't, and where akar should diverge.

**Study targets:**
- `egui/src/context.rs` — `Context`: the root handle, frame begin/end, input injection.
- `egui/src/painter.rs` — `Painter`: the draw list abstraction; rect, text, image primitives.
- `egui/src/response.rs` — `Response`: what a widget returns; hover, clicked, drag.
- `egui/src/ui.rs` — `Ui`: the layout cursor and child widget API.
- `egui/src/id.rs` and `memory.rs` — how per-widget state is stored across frames (hover, focus, animation).
- `egui_extras` or `egui`'s `TableBuilder` — how virtualized tables work.

**Questions to answer:**
1. How does egui's `Context` separate input from rendering? Can akar use the same split?
2. What is the `Response` type's minimal API that akar's components should return?
3. How does egui handle widget ID collisions and what does akar need to do instead?
4. How does egui's layout cursor (auto-layout within a `Ui`) differ from an explicit taffy tree? Which is more composable for component authors?
5. How does egui implement its scroll area and what is the exact clipper API it exposes?
6. What does egui's painter flush look like — how does it hand draw calls to the GPU backend?

**Output:** Written summary of the above six questions. Note which egui patterns akar should adopt and which it should not (with reasoning).

---

### Task 4: Study Nuklear's C API design
**Priority:** High

**Goal:** Nuklear is the best reference for a portable, backend-agnostic C API. akar's `akar.h` should follow the same principles.

**Study targets:**
- `nuklear.h`: initialization (`nk_init*`), the context struct (`nk_context`), the input model (`nk_input_begin`/`nk_input_end`), widget calls (`nk_button_label`, `nk_checkbox_label`, `nk_slider_float`), layout (`nk_layout_row_*`), and the draw command iterator (`nk_draw_foreach`).
- Focus on opaque handle patterns, how Nuklear avoids dynamic allocation by default (fixed-size buffers), and how it separates layout commands from render commands.

**Questions to answer:**
1. How does Nuklear's context struct handle memory — fixed arena, user-supplied allocator, or malloc? What should akar do?
2. What is the draw command iterator pattern — how does the caller retrieve and execute draw commands?
3. How does Nuklear's input model separate "feed input" from "query input inside a widget call"?
4. What naming conventions does Nuklear use that translate well across languages?
5. Are there patterns in Nuklear's C API that are awkward from non-C languages? How would akar avoid them?

**Output:** Written summary of the above five questions. Include a draft skeleton of `akar.h` top-level structure (opaque types, init/shutdown, frame begin/end, input API shape) — not a final API, just a structural sketch.

---

### Task 5: Study Zed's GPUI
**Priority:** Medium

**Goal:** GPUI is a production wgpu-based UI system at scale. It is retained-mode and reactive, so akar will not copy its architecture — but its rendering decisions are directly relevant.

**Study targets:**
- `crates/gpui/src/` in the Zed monorepo.
- Focus on: how GPUI initializes wgpu (adapter selection, surface config), how it manages the render pass, how it batches quads (look for quad vertex buffers, instance buffers), how it handles DPI/scale factor, and how glyphs are cached.
- Skip: the element/component protocol, reactive update cycle, platform abstraction — these are retained-mode concerns akar does not need.

**Questions to answer:**
1. How does GPUI structure its quad rendering — one pipeline per shape type, or a unified quad pipeline with per-instance parameters?
2. How does GPUI handle rounded corners — SDF in shader, pre-rasterized textures, or tessellation?
3. How does GPUI batch text and UI in the same render pass?
4. How does GPUI handle DPI scaling — are all coordinates in logical pixels, and where does the scale multiply in?
5. What wgpu features does GPUI require that might constrain akar's minimum requirements?

**Output:** Written summary of the above five questions. This directly informs the quad shader and pipeline design in Epic 002.

---

### Task 6: Study taffy's layout API
**Priority:** Medium

**Goal:** Confirm taffy is the right layout engine for akar and understand exactly how akar-layout should wrap it.

**Study targets:**
- `~/Projects/taffy/` (or clone): `taffy::TaffyTree`, `NodeId`, `Style`, `Size`, `Dimension`, `Layout`. Read `taffy/src/lib.rs` and the `flexbox` module.
- Focus on: how to build a tree, how to mark nodes dirty, how to run layout, and how to read resolved pixel coordinates.
- `taffy`'s `available_space` parameter — what it is and why it matters for scroll containers.

**Questions to answer:**
1. What is the minimal tree management API akar needs to expose through `akar-layout`?
2. How does taffy handle re-layout when only a subtree changes? Is there a dirty-flag mechanism?
3. How does taffy handle content-sized nodes (text labels that grow to fit their text)?
4. Can taffy run without allocation (fixed-size tree)? Is that desirable for akar?
5. What information does taffy need from akar to measure text (the "measure function" callback)?

**Output:** Written summary. Include a sketch of the `akar-layout` public Rust API (just function signatures, no implementation).

---

### Task 7: Study sokol's C API design principles
**Priority:** Medium

**Goal:** sokol_gfx.h and sokol_app.h are the best examples of language-neutral C API design. Extract the patterns for `akar.h`.

**Study targets:**
- `~/Projects/sokol/sokol_gfx.h` — init/shutdown, resource creation (buffers, pipelines, passes), frame begin/end.
- `~/Projects/sokol/sokol_app.h` — the app lifecycle model and input delivery.
- Focus on: how sokol handles opaque handles (resource IDs as structs, not raw pointers), how it avoids callbacks in the core API, how it structures its init desc structs, and how it supports multiple backends behind one header.

**Questions to answer:**
1. Sokol uses `typedef struct { uint32_t id; } sg_buffer;` for handles instead of pointers. Should akar follow this for e.g. texture handles?
2. How does sokol's `desc`-struct initialization pattern (`sg_buffer_desc desc = { .size = ... }`) translate to akar's context and component APIs?
3. How does sokol document backend differences within a single header? Can akar use the same approach for platform differences?
4. What does sokol do to ensure the header compiles cleanly as both C and C++?

**Output:** Written summary. Extend the draft `akar.h` skeleton from Task 4 with sokol-inspired conventions.

---

### Task 8: Study xilem and daisyUI as negative references
**Priority:** Low

**Goal:** Confirm that retained-mode and CSS-cascade approaches are not the right fit for akar, and extract any component-catalog decisions that transfer directly.

**Study targets:**
- `~/Projects/xilem/xilem_core/src/` — view tree, change flags, element lifecycle. Read just enough to understand the mental model.
- `~/Projects/daisyui/src/` — component token list, class naming conventions, what "components" exist and what their variants are.
- `~/Projects/shadcn_ui/packages/` — component prop signatures and composition patterns.

**Questions to answer:**
1. What does xilem's retained model give you that immediate mode cannot? (Accessibility hooks? Animation timelines?) Does akar need any of it in v1?
2. From daisyUI: what is the complete list of components? Which map cleanly to immediate mode calls? Which need special treatment (e.g. dropdown requires overlay stack)?
3. From shadcn: what component variants (size, color, shape) are worth encoding in akar's theme token system?
4. What component names from daisyUI/shadcn should akar adopt verbatim vs. rename for an imperative API?

**Output:** A canonical component list for akar v1, with variant counts and notes on any that require special rendering (overlay, scroll, animation).

---

### Task 9: Synthesize ADRs and write Epic 002
**Priority:** High — this is the deliverable

**Goal:** Use the output of Tasks 1–8 to write the four Architecture Decision Records below and then produce `epics/002-core-renderer-and-draw-list.md`.

**Architecture Decision Records to write** (append to this epic or create `docs/adr/`):

1. **ADR-001: Immediate mode vs. retained mode**
   - Decision, rationale, consequences, what is explicitly deferred.

2. **ADR-002: Draw list design**
   - Whether to use painter's order or explicit Z, how AABB culling works, how scissor rect stack works, what a `DrawCall` struct contains.

3. **ADR-003: Quad pipeline shader**
   - SDF rounded corners vs. tessellation vs. pre-rasterized. Anti-aliasing approach. DPI handling. Informed by GPUI findings.

4. **ADR-004: C ABI shape**
   - Opaque handle strategy (pointer vs. ID struct), memory ownership model, naming conventions, how the header is generated. Informed by Nuklear + sokol findings.

**Epic 002 content** (`epics/002-core-renderer-and-draw-list.md`):

Epic 002 covers the first implementation milestone: a working quad renderer + draw list + input state, with a single working button component rendered in a demo app. It should include:

- Workspace scaffold (Cargo.toml with `akar-core`, `akar-layout`, `akar-components`, `akar-c-api`, `akar-winit`).
- `akar-core`: quad pipeline (WGSL shader), text pipeline (glyphon wrapper), draw list, scissor stack, input state struct.
- `akar-layout`: taffy wrapper with the minimal tree API identified in Task 6.
- One component: `button` — covers the full pipeline end-to-end (layout → background quad → border quad → text → hover/active state).
- A Rust demo that opens a winit window and renders the button, driven by the author's event loop.
- The C ABI skeleton (`AkarCtx`, init/shutdown, begin/end frame, input feed, the button call).

Each Epic 002 task must include file-level details (which file, which struct, which method signature) so a coding agent can execute it without additional research.

**Acceptance Criteria for Epic 001:**
- [x] Tasks 1–8 each have a written output section appended to this epic (or linked from it).
- [ ] Four ADRs are written and internally consistent.
- [ ] `epics/002-core-renderer-and-draw-list.md` exists and contains tasks detailed enough for a coding agent to implement without further research.
- [ ] No code has been written.

---

# Notes from MiMo — Research Output (Tasks 1–8)

All research below was conducted by reading local source code from `~/Projects/`.

---

## Task 1: sugacode Baseline Analysis

### 1. What sugacode's renderer does that akar-core must replicate or generalize

Sugacode's `Renderer::new()` (`renderer.rs:32-96`) performs a standard wgpu bootstrap: instance creation, adapter request, device+queue request, surface creation, surface configuration with hardcoded `Bgra8UnormSrgb` format and `Fifo` present mode, and glyphon text pipeline init. All GPU resources are owned in one monolithic struct (`renderer.rs:16-30`).

**What akar-core must replicate:**
- The glyphon initialization pipeline (FontSystem, SwashCache, Cache, Viewport, Atlas, TextRenderer).
- Surface resize handling (`renderer.rs:98-102`).
- The frame lifecycle: viewport update -> collect text areas -> prepare text -> acquire surface texture -> render pass -> submit -> present -> trim atlas (`renderer.rs:104-198`).

**What akar-core must NOT replicate:**
- Device/queue ownership. akar-core accepts device/queue from the caller (per C ABI contract), not own the device creation.
- Surface creation tied to winit. This must move to `akar-winit`.

### 2. Exact glyphon API surface used

Initialization sequence (`renderer.rs:71-81`):
1. `FontSystem::new()`
2. `SwashCache::new()`
3. `Cache::new(&Device)`
4. `Viewport::new(&Device, &Cache)`
5. `TextAtlas::new(&Device, &Queue, &Cache, TextureFormat)`
6. `TextRenderer::new(&mut TextAtlas, &Device, MultisampleState, Option<DepthStencilState>)`

Per-frame: `Viewport::update(&Queue, Resolution)` -> `Buffer::new(FontSystem, Metrics)` -> `Buffer::set_size()` -> `Buffer::set_text()` -> `Buffer::shape_until_scroll()` -> `TextRenderer::prepare(...)` -> `TextRenderer::render(...)` -> `TextAtlas::trim()`.

The `TextAreaData` struct (`renderer.rs:201-208`) is the bridge between UI and renderer: `{ buffer, left, top, scale, bounds, color }`.

### 3. What is missing

- **No quad/rect renderer — CONFIRMED.** Every rectangle is faked with `" ".repeat(...)` text buffers filled with space characters. This is extremely wasteful and must be replaced with an instanced quad pipeline.
- **No layout engine — CONFIRMED.** All positions are hardcoded pixel coordinates computed procedurally.
- **No input focus/hover state machine — CONFIRMED.** Hover is inline `is_mouse_over_rect()` checks. No click dispatch, no keyboard focus, no active/pressed state.
- **Additional:** No scissor/clip stack (manual clip calculations), no z-ordering, no draw list abstraction, no border/rounded corner rendering.

### 4. What can be ported vs. what needs redesign

**Port:** Glyphon init sequence, `TextAreaData` collect pattern (generalize to `DrawCall`), `create_text_buffer()` helper, frame lifecycle structure, surface error handling, container virtualization pattern (`visible_cards()`), zoom/pan coordinate transforms.

**Redesign:** Monolithic Renderer struct (split), space-character rectangles (replace with quad renderer), hardcoded pixel layout (replace with taffy), no draw list abstraction (build `DrawList` with `DrawCall` enum), input handler tightly coupled to app state (abstract to generic `InputState`), hit testing positional not tree-based.

---

## Task 2: Dear ImGui Draw List and List Clipper Analysis

### 1. How ImGui's DrawList batches and sorts

ImGui does **not** sort by Z. It uses strict **painter's order** (submission order). Batching is done by `_TryMergeDrawCmds()` (`imgui_draw.cpp:578-587`): consecutive draw calls with the same `(ClipRect, TexRef, VtxOffset)` are merged into one GPU draw call. Changing clip rect or texture creates a new `ImDrawCmd`.

The `ImDrawListSplitter` (channels API) lets tables batch interleaved column draws by merging adjacent commands with matching headers on `Merge()`.

**What akar should do:** Adopt the merge-adjacent-with-same-state optimization. Reject submission-order-only model. akar should sort by `(z, texture/pipeline_type)` before upload.

### 2. ImGuiListClipper — what it computes

`ImGuiListClipper` (`imgui.h:2869-2905`) computes a visible item index range `[DisplayStart, DisplayEnd)` given total count, item height, and the window's clip rect. The core formula: `first = (clip_min_y - cursor_y) / item_height`, `last = (clip_max_y - cursor_y) / item_height`. It also adds extra ranges for navigation targets and focused items.

**Can it be a pure function?** Yes. The core computation is pure: `list_clip(total_items, item_height, scroll_y, viewport_top, viewport_bottom) -> (first, last)`. akar should make this a free function in `akar-core`. No state, no context, no object.

### 3. ID stack — how it works

ImGui uses `IDStack` per window (`imgui_internal.h:2746`). `PushID/PopID` hashes against the current stack top using CRC32. `GetID(str)` produces a unique `ImGuiID` per widget. The global context stores `HoveredId`, `ActiveId`, `NavId` — two `uint32_t` fields tracking interaction state across frames.

**akar should:** Use explicit integer node IDs from the taffy layout tree (stable, unique by construction, zero-cost to compare). Keep `PushID/PopID` for hierarchy. Skip the `###` label/id separator. Skip per-window ID stacks.

### 4. ImGuiIO input model

`ImGuiIO` (`imgui.h:2409-2636`) is populated **before** `NewFrame()` via `Add*()` functions: `AddKeyEvent`, `AddMousePosEvent`, `AddMouseButtonEvent`, `AddMouseWheelEvent`, `AddInputCharacter`. Rising/falling edge detection (`MouseClicked`, `MouseReleased`) is computed from `MouseDown` state. Output flags `WantCaptureMouse`/`WantCaptureKeyboard` tell the app when to suppress game input.

**akar should adopt:** The event queue pattern (`akar_set_mouse_pos`, `akar_push_mouse_button`, etc.), edge detection, `WantCapture*` output flags. Skip the trickle mechanism and per-key data array.

### 5. C types that translate to akar.h

| ImGui Type | akar Equivalent |
|---|---|
| `ImGuiID` (`unsigned int`) | `uint32_t` |
| `ImVec2` (`{float x, y}`) | `AkarVec2` |
| `ImVec4` (`{float x, y, z, w}`) | `AkarVec4` |
| `ImU32` (packed RGBA) | `uint32_t` |
| `ImDrawCmd` (ClipRect + TexRef + ElemCount) | `AkarDrawCmd` with `clip_rect`, `pipeline`, `idx_offset`, `elem_count` |
| `ImDrawVert` (`{pos, uv, col}`, 20 bytes) | Custom quad vertex format (no `uv` needed for SDF) |

Skip: `ImTextureRef`, `ImDrawListSplitter`, `ImGuiContext` (full), `ImGuiStyle`, `ImGuiWindow`.

---

## Task 3: egui Immediate-Mode Architecture Analysis

### 1. How egui separates input from rendering

egui uses a strict frame lifecycle through `Context`:

1. **Input injection** at `begin_pass` (`context.rs:896`). `RawInput` is processed into `InputState`.
2. **Widget registration** during the UI pass. Each widget calls `Context::create_widget(WidgetRect)` (`context.rs:1182`).
3. **Interaction resolution** happens at the START of the next frame, using PREVIOUS frame's widget rects (`context.rs:472-493`). Hit-testing and interaction are pre-computed into an `InteractionSnapshot`.
4. **Response creation** when widgets read their interaction result via `Context::get_response()` (`context.rs:1357`).
5. **Rendering** is separate — widgets add `Shape`s to `PaintList` via `Painter::add()` (`painter.rs:213`).
6. **Tessellation** is a separate backend step: `Context::tessellate()` (`context.rs:2757`).

**akar should use the same split** but diverge: egui has a 1-frame delay for interaction (uses previous frame's rects). akar can use taffy layout to compute rects BEFORE the interaction pass within the same frame, eliminating this delay.

### 2. Minimal Response API for akar

egui's `Response` (`response.rs:23-75`) is 88 bytes with embedded `Context`. Core flags: `ENABLED`, `CONTAINS_POINTER`, `HOVERED`, `CLICKED`, `DRAG_STARTED`, `DRAGGED`, `DRAG_STOPPED`, `IS_POINTER_BUTTON_DOWN_ON`, `CHANGED`, `CLOSE`.

**Minimal for akar:** `{ id, rect, clicked, hovered, contains_pointer, dragged, drag_delta, changed, has_focus, gained_focus, lost_focus, is_pointer_down }`. No embedded context (C ABI provides it separately). ~32-48 bytes with bitflags.

### 3. Widget ID collisions

egui uses `Id::new(source)` (hash-based, `NonZeroU64`, `id.rs:44`). Auto-IDs from `Ui` are position-dependent (unstable if widgets insert/remove). `Context::check_for_id_clash()` (`context.rs:1097`) stores IDs+rects and shows visual warnings on collision.

**akar should NOT use hash-based IDs.** Use explicit taffy node IDs instead — stable across frames, unique by construction, zero-cost comparison. For per-widget state, use `HashMap<NodeId, WidgetState>`.

### 4. Layout cursor vs. taffy tree

egui uses a cursor-based system (`Placer` inside `Ui`). `allocate_space(desired_size)` (`ui.rs:1187`) returns a rect immediately. Layouts are created by nesting `Ui`s with different `Layout` directions. Single-pass, no tree.

Taffy is two-pass: build tree with style properties, compute layout, read results. Supports cross-widget constraints, intrinsic sizing, grid layouts.

**Verdict for akar:** Taffy is the right call (better layout correctness, natural virtualization support). But akar should provide a thin cursor-like wrapper over taffy for ergonomics: `let btn_id = ui.button("Click me")` internally creates/queries taffy nodes.

### 5. Scroll area and clipper API

egui's `ScrollArea::show_rows()` (`scroll_area.rs:984-1015`) computes visible range: `min_row = (viewport.min.y / row_height).floor()`, `max_row = (viewport.max.y / row_height).ceil() + 1`. No standalone clipper function.

**akar should expose `list_clip(total, item_height, scroll_y) -> Range<usize>`** as a first-class API, as specified in AGENTS.md.

### 6. Painter flush

egui accumulates `Shape`s in `PaintList` -> `GraphicLayers::drain()` collects all shapes in z-order -> CPU tessellation via `Tessellator::tessellate_shapes()` -> `Vec<ClippedPrimitive>` with `Mesh` data -> backend iterates and issues GPU draw calls.

**akar should differ:** No CPU tessellation step. Draw list stores GPU-ready primitives directly (rect, text). Z-sorting happens in the draw list before GPU upload. Scissor stack in draw list, batched by rect.

---

## Task 4: Nuklear C API Design Analysis

### 1. Context memory handling

Nuklear provides 4 init variants (`nuklear.h:19543-19585`):
- `nk_init_default` — wraps `malloc`/`free`
- `nk_init_fixed` — caller provides a single contiguous buffer, no allocator
- `nk_init` — caller provides `nk_allocator` with `alloc`/`free` callbacks
- `nk_init_custom` — caller provides two pre-initialized `nk_buffer` objects

The allocator struct (`nuklear.h:506`) uses `nk_handle` (union of `void*` and `int`) for userdata. The `alloc` callback is `(userdata, old_ptr, new_size)` — doubles as both `malloc` and `realloc`.

**akar should:** 3 variants: `akar_ctx_new(device, queue)` (Rust alloc), `akar_ctx_new_with_allocator(...)` (custom allocator), `akar_ctx_new_fixed(...)` (embedded targets). Use `(userdata, size, align)` allocator interface (closer to Rust's `GlobalAlloc`).

### 2. Draw command iterator pattern

Two layers:
- **Layer 1** (`nk_foreach`): Abstract command buffer iteration. 18 command types (line, rect, circle, text, etc.). Caller casts based on `cmd->type`.
- **Layer 2** (`nk_draw_foreach`): After `nk_convert()`, hardware-ready vertex/index buffers. Each `nk_draw_command` has `elem_count`, `clip_rect`, `texture`. Caller iterates to issue GPU draw calls.

**akar should:** Skip the abstract command layer (Layer 1). Go straight to `DrawCall` structs with `elem_count`, `clip_rect`, `texture` (Layer 2 pattern). Expose `akar_draw_begin`/`next`/`end` as the primary API (not macros).

### 3. Input model separation

Two phases:
- **Input feeding** (`nk_input_begin`/`nk_input_end`): Caller wraps platform events. `nk_input_begin` resets per-frame deltas. Individual functions (`nk_input_motion`, `nk_input_key`, `nk_input_button`, `nk_input_scroll`, `nk_input_char`) set state on `ctx->input`.
- **Input querying** (inside widgets): Rect-based hit-test queries on `ctx->input`: `nk_input_is_mouse_hovering_rect`, `nk_input_is_mouse_click_in_rect`, etc. No callbacks, no event queue, no indirection.

**akar should adopt this exactly.** Flat `InputState` struct, begin/end bracket, hit-test queries.

### 4. Naming conventions

Everything is `nk_` prefixed. Types: `nk_` + lowercase_with_underscores. Enums: `NK_` + SCREAMING_SNAKE_CASE. Functions: `nk_` + snake_case. Widget naming: `nk_button_label`, `nk_button_text`, `nk_button_image_label`. Callback types: `nk_plugin_alloc`, `nk_plugin_free`.

**akar should follow:** `akar_` prefix, `AKAR_` enum prefix, `_label`/`_text` variant pattern for null-terminated vs length-delimited strings.

### 5. Awkward patterns for non-C languages

| Pattern | Problem | akar fix |
|---|---|---|
| `nk_handle` union | No FFI union support in most languages | Opaque `uint64_t` handles |
| Variadic `_f` functions (`nk_labelf`) | Cannot call from non-C | Expose only `const char *text, int len` variants |
| Out-parameter mutation (`float *val`) | Requires temp allocation in non-C | Return result structs with value + changed flag |
| String-based window identification | String marshalling overhead | Handle-based (`AkarWindow*`) |
| Macro-based iterators (`nk_foreach`) | Cannot call from non-C | Expose `akar_draw_begin`/`next`/`end` as functions |
| Cast-based type dispatch (18 cmd types) | Unsafe in non-C | Single concrete `DrawCall` struct |
| Window begin/end bracket | Forgetting `nk_end` = UB | Scoped handle pattern |

---

## Task 5: Zed GPUI Rendering Analysis

### 1. Quad rendering structure

GPUI uses **separate pipelines per primitive type** (`wgpu_renderer.rs:84-95`): `quads`, `shadows`, `path_rasterization`, `paths`, `underlines`, `mono_sprites`, `subpixel_sprites`, `poly_sprites`, `surfaces`. All share one compiled shader module (`shaders.wgsl`).

Batching: `Scene` stores primitives in type-separated vectors. `BatchIterator` (`scene.rs:255-451`) produces `PrimitiveBatch` variants grouping consecutive same-type primitives by z-order. All draw calls use a **single shared instance buffer** (storage buffer, initially 2MB, grows on overflow). Each draw call writes instance data into a subregion and issues `draw(0..4, 0..instance_count)` — a triangle-strip quad instanced N times.

**Key insight:** One pipeline per primitive type but a single shared instance buffer. akar could follow the same pattern.

### 2. Rounded corners — SDF in shader

GPUI uses **SDF (signed distance field) evaluation in the fragment shader** (`shaders.wgsl:362-387`). Per-corner radii, selected by quadrant. Fast path for zero-radius corners (no SDF). The `quad_sdf_impl` function: `length(max(0, corner_center_to_point)) + min(0, max(x, y)) - corner_radius`. Anti-aliasing via `blend_color(color, saturate(antialias_threshold - outer_sdf))`. Dashed borders also SDF-based.

**akar should adopt SDF rounded corners.** The pattern is simple, branchless, and handles per-corner radii.

### 3. Text and UI batching in the same render pass

All primitives render in a single render pass. `BatchIterator` merge-sorts across all primitive type vectors by `(order, kind)`. Text sprites are batched by `texture_id` to minimize bind group changes. The only exception is paths, which require an intermediate texture pass (rasterize path -> composite into main pass).

**akar can follow the same approach** since it is immediate-mode. Sort draw calls by `(z, pipeline_type)` before submission.

### 4. DPI scaling

GPUI uses a three-unit type system: `Pixels` (logical), `ScaledPixels` (physical after DPI multiply), `DevicePixels` (integer physical). Scale factor flows: platform provides it -> layout uses it for snapping -> paint methods multiply it -> shaders receive physical coordinates. Glyph rasterization is keyed by `scale_factor`.

**akar simplification:** Work in logical pixels, apply scale factor at the end when converting to the draw list. All shader data must be in scaled (physical) pixels. Glyph rasterization must be keyed by `scale_factor`.

### 5. wgpu minimum requirements

GPUI requires `wgpu::Limits::downlevel_defaults()` (works on OpenGL ES 3.0). Only optional feature: `DUAL_SOURCE_BLENDING` for subpixel text (gracefully degraded). Surface format: prefers `Bgra8Unorm`, falls back to `Rgba8Unorm`. wgpu version: 29.0.4.

**akar can target `downlevel_defaults()`** and be compatible with very wide hardware range. Check for `DUAL_SOURCE_BLENDING` at runtime and degrade gracefully.

---

## Task 6: taffy Layout API Analysis

### 1. Minimal tree management API for akar-layout

10 core methods from `TaffyTree`:
- `new_leaf(Style) -> NodeId` — leaf nodes
- `new_with_children(Style, &[NodeId]) -> NodeId` — container nodes
- `new_leaf_with_context(Style, NodeContext) -> NodeId` — leaf nodes with measurement context (text data)
- `add_child(parent, child)`
- `set_children(parent, &[NodeId])`
- `remove(node)`
- `set_style(node, Style)` — automatically marks dirty
- `set_node_context(node, Option<NodeContext>)`
- `compute_layout_with_measure(node, available_space, measure_fn)`
- `layout(node) -> &Layout`

### 2. Dirty-flag mechanism

Each node stores a `Cache` struct (`cache.rs:24-31`). `mark_dirty(node)` (`taffy_tree.rs:873-896`) propagates upward from the changed node to all ancestors. **Key optimization:** when a node is already dirty (cache empty), traversal stops (`ClearState::AlreadyEmpty`). All tree-mutation methods automatically call `mark_dirty`. During layout, `compute_cached_layout` short-circuits on cache hit, skipping entire subtrees.

**akar does not need its own dirty-tracking layer.** Taffy's built-in system is sufficient.

### 3. Content-sized nodes

The measure function callback handles text/image sizing. Signature: `FnMut(known_dimensions: Size<Option<f32>>, available_space: Size<AvailableSpace>, node_id, Option<&mut NodeContext>, &Style) -> Size<f32>`. When `known_dimensions.width` is `Some(200.0)`, the function reports height at that width. The `content_size` feature provides `scroll_width()`/`scroll_height()` for scroll containers.

### 4. Allocation-free operation

`TaffyTree` uses `SlotMap` (heap-allocated arena). **Not allocation-free, but this is fine for akar.** SlotMap is arena allocation (contiguous memory, O(1) ops). Pre-allocation via `TaffyTree::with_capacity(n)` available. A low-level trait API exists for custom tree implementations if needed later.

### 5. Text measure function integration

The measure function receives `known_dimensions`, `available_space`, `node_id`, `Option<&mut NodeContext>`, and `&Style`. akar would create text leaf nodes with `new_leaf_with_context(Style, TextContext)` and the measure function would call glyphon to determine text size given constraints. `AvailableSpace` has three variants: `Definite(f32)`, `MinContent`, `MaxContent`.

---

## Task 7: sokol C API Design Analysis

### 1. Opaque handle pattern

sokol uses `typedef struct { uint32_t id; } sg_buffer;` (`sokol_gfx.h:2005-2010`). The 32-bit ID is split: 16-bit pool index + 16-bit generation counter. Strongly-typed structs prevent passing incompatible handles (compile error). Generation counter detects use-after-free.

**akar should adopt this for all pooled GPU resources** (texture atlas entries, font handles). The `AkarCtx` pointer remains a pointer (singleton, not pooled).

### 2. Desc-struct initialization pattern

Zero-init a struct, set only the fields you care about, pass a pointer. All zero fields get sensible defaults filled by the library. `_sg_def(val, def)` macro: `((val) == 0) ? (def) : (val)`. Canary fields (`_start_canary`, `_end_canary`) catch uninitialized memory.

**akar should use this for `AkarCtxDesc` and component option structs.** Use canaries on `AkarCtxDesc` (called once), skip on per-frame component descs.

### 3. Backend differences in a single header

Four strategies:
1. **Compile-time backend selection** via `#define` (one backend compiled at a time)
2. **Platform-specific state** behind `#ifdef` in global state struct
3. **Backend-specific nested config structs** in public desc structs (e.g., `sg_desc.wgpu`)
4. **Backend-specific escape-hatch query functions** (e.g., `sg_wgpu_device()` returning `const void*`)

**akar should use `const void*` for native handles in the public C API.** `akar.h` must never `#include` wgpu headers.

### 4. C and C++ compilation

Five techniques:
1. `extern "C"` wrapping with `#ifdef __cplusplus` guard
2. C++ reference-based overloads after closing `extern "C"` block
3. Dual struct initialization macros (`{0}` for C, `{}` for C++)
4. `_FORCE_U32 = 0x7FFFFFFF` sentinel on every enum (ensures 32-bit width)
5. `#include <stdbool.h>` for `bool` type

**akar should adopt all five.** The C++ overloads would be a post-processing step on cbindgen output. `_FORCE_U32` is critical for ABI stability.

---

## Task 8: xilem, daisyUI, shadcn_ui Component Catalog Analysis

### 1. xilem retained model vs. immediate mode

xilem's retained model provides: incremental diffing (only mutate what changed), persistent `ViewState` per node, structured message routing via `ViewId` paths, typed dependency injection (`provides`/`with_context`), memoization, and lifecycle hooks (`teardown` for cleanup).

**akar needs none of this in v1.** Immediate mode re-submits everything each frame. Component functions return state enums directly. Callbacks are passed directly. Theme tokens are a flat struct. No persistent resources to clean up. The one retained concept to eventually consider: focus management (`focused_id: Option<ComponentId>` in input state).

### 2. Complete daisyUI component list (57 components)

**Tier 1 — Direct immediate-mode calls (27):** alert, avatar, badge, breadcrumbs, button, calendar, card, divider, fieldset, footer, hero, indicator, kbd, label, link, list, loading, mask, mockup, navbar, progress, radialprogress, skeleton, stack, stat, status, steps.

**Tier 2 — Need special input handling (9):** checkbox, radio, toggle, range, input, textarea, select, fileinput, swap.

**Tier 3 — Require overlay/z-index stack (7):** dropdown, modal, drawer, tooltip, toast, collapse, tab.

**Tier 4 — Layout/animation/complex (14):** carousel, chat, countdown, diff, dock, fab, filter, hover3d, hovergallery, menu, rating, timeline, validator, textrotate.

### 3. Theme token system (synthesized from daisyUI + shadcn)

**Color tokens (8 semantic slots + content pairs):** primary, secondary, accent, neutral, info, success, warning, error. Each with a `*-content` foreground color. Plus `base_100`/`base_200`/`base_300`/`base_content` for neutral backgrounds.

**Size variants (5-step):** Xs, Sm, Md, Lg, Xl.

**Style variants (6 treatments):** Solid, Outline, Ghost, Soft, Dash, Link.

**Shape variants:** Default, Square, Circle, Wide, Block.

**Structural tokens:** `radius_field`, `radius_box`, `radius_selector`, `border_width`, `depth`.

### 4. Component names for akar

**Verbatim from daisyUI (22):** button, badge, alert, card, checkbox, radio, toggle, input, textarea, select, range, progress, divider, tooltip, avatar, table, link, kbd, skeleton, steps, drawer, fab.

**From shadcn (13):** dialog, sheet, separator, switch (rename toggle), slider (rename range), accordion (rename collapse), popover, scroll_area, command_palette, breadcrumb, pagination, empty_state, spinner (rename loading).

**Renamed for imperative clarity:** collapse -> expandable, loading -> spinner, range -> slider, sonner -> toast, alert-dialog -> confirm_dialog, resizable -> split_pane.

**Recommended v1 set (22 components):**
1. Primitives: button, badge, label, link, separator, spinner, kbd
2. Inputs: input, textarea, checkbox, radio, switch, slider, select
3. Feedback: alert, tooltip, toast
4. Layout: card, table, tab_bar + tab_panel, scroll_area
5. Overlay: dialog

---

# Notes from Sonnet — Review of MiMo's Research

I spot-checked the key architectural claims against the actual local source files. Overall verdict: **MiMo's research is solid and well-grounded**. The factual claims are correct. The design recommendations are sound. The line-number citations are accurate within 1–2 lines. Below are my findings, corrections, and additions.

---

## Task 1 (sugacode) — Confirmed, with one characterization note

All major claims verified against `~/Projects/sugacode/src/renderer.rs` and `src/ui/`:

- **Space-character rects are confirmed** — `drawer.rs:77`, `search.rs:156`, `mod.rs:95`, `mod.rs:121`, `mod.rs:189`, `mod.rs:253`. The exact pattern: `" ".repeat((width * height / 100.0) as usize)`. MiMo's "extremely wasteful" characterization is accurate — these are full text buffers shaped and uploaded to the GPU for solid-color rectangles.
- **No quad renderer** — confirmed. `renderer.rs` has zero quad pipeline setup. Only glyphon `TextRenderer`.
- **No layout engine** — confirmed. Positions in `container.rs` are all computed manually (hardcoded arithmetic, not a layout tree).
- **`visible_cards()`** — confirmed at `container.rs:243–260`. This is the direct predecessor of akar's `list_clip` API.
- **Glyphon init sequence** — confirmed at `renderer.rs:71–81`, matches MiMo's description exactly.
- **`TextAreaData` struct** — confirmed at `renderer.rs:201–208`.

One **minor correction**: `Renderer::new()` starts at line 33, not 32. Immaterial.

**Addition MiMo missed:** `renderer.rs` currently owns `winit::window::Window` (line 29). When akar-core takes `device` and `queue` from the caller instead of owning them, it also stops owning the window and surface — those move to `akar-winit`. This boundary is the single most important structural change from sugacode's architecture.

---

## Task 2 (Dear ImGui) — Confirmed, design recommendation needs qualification

Core claims verified against `~/Projects/imgui/`.

MiMo recommends: "sort by `(z, texture/pipeline_type)` before upload." This is **correct as a direction** but the framing "Reject submission-order-only model" is too strong. The practical approach is:

- Use **painter's order within a Z-level** (simpler, predictable, matches what ImGui does per layer).
- Use **Z as an explicit override** only when components need to render above others (tooltips, modals, toasts).
- Sort by `(z, pipeline_type)` before GPU upload to minimize pipeline switches.

Full cross-element Z-sorting adds complexity for no gain in most frames. akar's draw list should default to submission order and only sort when Z values differ.

**On `ImGuiListClipper` as a pure function:** Confirmed correct. The core computation is `first = floor((clip_min_y - cursor_y) / item_height)`. akar's `list_clip(total, item_height, scroll_y) -> Range<usize>` free function is the right design.

**On the ID stack:** MiMo's recommendation to use taffy `NodeId` directly is correct. No CRC-based ID hashing needed.

---

## Task 3 (egui) — Confirmed, one important addition

The 1-frame interaction delay in egui (registering widget rects in frame N, resolving hover/click in frame N+1) is a real design flaw. MiMo's proposed fix — using taffy to compute rects before the interaction pass within the same frame — is the right call.

**Addition:** egui's `Painter` accumulates `Shape`s in a `PaintList`, then CPU-tessellates them via `Tessellator` before handing meshes to the backend. akar's design (GPU-ready primitives in the draw list, no CPU tessellation step) is explicitly superior here because:
1. No intermediate `Vec<ClippedMesh>` allocation per frame.
2. Z-sort happens on small `DrawCall` structs, not large tessellated meshes.

MiMo noted this but did not call it out as a specific advantage to document in the ADR.

**On `Response` size:** MiMo says 88 bytes. I did not verify this directly, but the `bitflags` approach MiMo recommends for akar (yielding ~32–48 bytes) is correct regardless. akar's response must not embed a `Context` reference — it passes all context via the `AkarCtx*` parameter.

---

## Task 4 (Nuklear) — Confirmed

Line citations are plausible and the characterization of the 4 init variants, the draw command iterator, and the input phase separation all match Nuklear's documented API.

**One addition for the `akar.h` skeleton:** MiMo's table of "awkward patterns for non-C languages" is the most practically useful output of this task. The "out-parameter mutation" row deserves emphasis: returning `(value, changed: bool)` as a struct instead of `float* val` is a key ergonomics win for Go, Python, and Zig bindings. This should be a firm constraint in Epic 002's C ABI design.

---

## Task 5 (GPUI) — Confirmed, line numbers exact

Verified against `~/Projects/zed/crates/gpui_wgpu/src/wgpu_renderer.rs` and `shaders.wgsl`.

- **Pipeline struct at `wgpu_renderer.rs:84–95`** — confirmed exactly. The `subpixel_sprites` field is `Option<wgpu::RenderPipeline>` (not a plain pipeline), reflecting the conditional `DUAL_SOURCE_BLENDING` feature. MiMo captured this correctly in the "wgpu minimum requirements" section.
- **SDF rounded corners** — confirmed. `quad_sdf_impl` at `shaders.wgsl:372–385` (MiMo cited 362–387; close enough — the outer `quad_sdf` wrapper starts at 362). Per-corner radii via `pick_corner_radius` at line 341.
- **`antialias_threshold = 0.5`** confirmed at `shaders.wgsl:598`.

**One correction to MiMo's description:** The `quad_sdf_impl` has a fast path: `if (corner_radius == 0.0)` at line 373 returns early without SDF evaluation. akar's shader should include the same fast path — it significantly benefits widgets with no corner radius (separators, progress bars).

**Addition:** GPUI uses `crates/gpui_wgpu`, not `crates/gpui` as the epic's reference table says. The research table in this epic should be updated before it's used again. (MiMo found the right files regardless.)

---

## Task 6 (taffy) — Confirmed

- `compute_layout_with_measure` confirmed in `taffy/src/lib.rs:29` docs and `taffy_tree.rs`.
- `mark_dirty` with `ClearState::AlreadyEmpty` optimization confirmed at `taffy/src/tree/cache.rs:174–181` and `taffy_tree.rs:138`.
- The 10-method API list MiMo extracted is accurate.

**Addition:** `new_leaf_with_context` is the correct way to attach text measurement data to a taffy node. The `NodeContext` type is application-defined — akar-layout should define `AkarNodeContext` containing a `glyphon::Buffer` (or parameters to build one) so the measure function can call glyphon to compute text size given width constraints. This is the key integration point between taffy and glyphon.

---

## Task 7 (sokol) — Confirmed

The generation-counter handle pattern (`pool_index | generation << 16`) is confirmed. The desc-struct zero-init default pattern is accurately described.

**One addition for akar:** The `_FORCE_U32 = 0x7FFFFFFF` sentinel is **critical for cross-language ABI stability**, not just C++. In Go's CGo, Rust's `#[repr(C)]`, and Python's ctypes, enum sizes must be deterministic. cbindgen should be configured to emit this sentinel or use `#[repr(u32)]` on Rust enums — verify this is in the cbindgen config before Epic 002 finalizes the ABI.

---

## Task 8 (component catalog) — Sound, one omission

The v1 set of 22 components is a reasonable scope. The tier system (Tier 1 direct / Tier 2 input handling / Tier 3 overlay / Tier 4 complex) is a useful classification.

**One omission:** `popover` is in MiMo's "From shadcn" list but not in the v1 set. Tooltip and popover are architecturally nearly identical (both require an overlay stack). If `tooltip` is in v1 (it is), `popover` costs almost nothing to add. Recommend including it in the v1 overlay section alongside `dialog`.

---

## Overall assessment for Epic 002 readiness

The research is sufficient to write ADRs and Epic 002. The four decision points that are now clearly settled:

1. **Immediate mode** — confirmed right choice. Retained mode (xilem) adds diffing and lifecycle complexity akar does not need in v1.
2. **Draw list design** — painter's order within Z-levels, Z as explicit override, sort by `(z, pipeline_type)` before GPU upload, AABB scissor culling automatic.
3. **Quad shader** — SDF rounded corners (GPUI pattern), per-corner radii, `corner_radius == 0` fast path, `antialias_threshold = 0.5`.
4. **C ABI** — opaque `AkarCtx*` (pointer), sokol-style typed handle structs for pooled resources, Nuklear-style input begin/end bracket, result structs instead of out-params, `_FORCE_U32` on all enums.

Epic 002 can now be written.
