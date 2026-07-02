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
- [ ] Tasks 1–8 each have a written output section appended to this epic (or linked from it).
- [ ] Four ADRs are written and internally consistent.
- [ ] `epics/002-core-renderer-and-draw-list.md` exists and contains tasks detailed enough for a coding agent to implement without further research.
- [ ] No code has been written.
