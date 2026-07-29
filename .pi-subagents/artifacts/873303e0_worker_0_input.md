# Task for worker

## Task 8 — C ABI Coverage

**Epic:** epics/020-component-webpage-sample.md (read it fully for context)
**Key references:** DEVELOP.md, AGENTS.md, README.md

### What to implement

Add C-compatible representations and functions for the new components and styled APIs added in Tasks 3-7. The C API crate is `crates/akar-c-api/src/lib.rs`.

### Pattern to follow

Read the existing C API in `crates/akar-c-api/src/lib.rs` to understand the pattern:
- `#[repr(C)]` structs for all C-visible types
- `extern "C" fn` functions that take `*mut AkarCtx` and raw C parameters
- Node IDs are `u64` on the C side
- Strings are `*const c_char`, converted with `CStr::from_ptr` + `.to_str()`
- Enum variants are `u32` with const values
- Return structs are `#[repr(C)]`
- Null style pointers mean "use defaults"

### Types to add

#### Font/Text types (from Task 2)

```rust
#[repr(C)]
pub struct AkarFontFamily { pub value: u32 }  // 0=SansSerif, 1=Serif, 2=Monospace
#[repr(C)]
pub struct AkarFontWeight { pub value: u32 }  // 0=Normal, 1=Medium, 2=Semibold, 3=Bold
#[repr(C)]
pub struct AkarTextAlign { pub value: u32 }   // 0=Start, 1=Center, 2=End
```

#### TextStyle (C-compatible)

```rust
#[repr(C)]
pub struct AkarTextStyle {
    pub font_size: f32,          // 0.0 = use default
    pub line_height: f32,        // 0.0 = use default
    pub color: u32,              // 0 = use default
    pub font_weight: u32,        // 0xFF = use default
    pub font_family: u32,        // 0xFF = use default
    pub align: u32,              // 0xFF = use default
    pub wrap: u8,                // 0xFF = use default, 0 = no wrap, 1 = wrap
}
```

Use sentinel values (0.0 for floats, 0xFF for ints, 0xFF for wrap byte) to indicate "not set" since we can't use Option in C.

#### HeadingLevel

```rust
#[repr(C)]
pub struct AkarHeadingLevel { pub value: u32 }  // 0=H1, 1=H2, 2=H3, 3=H4
```

#### Component style types

```rust
#[repr(C)]
pub struct AkarCardLayout {
    pub direction: u32,    // 0=Column, 1=Row
    pub gap: f32,
    pub padding: f32,
    pub has_header: u8,
    pub has_footer: u8,
}

#[repr(C)]
pub struct AkarCardStyle {
    pub background: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub corner_radii: [f32; 4],
    pub shadow_blur: f32,
    pub shadow_spread: f32,
    pub shadow_color: u32,
    pub shadow_offset: [f32; 2],
    pub separator_color: u32,
}

#[repr(C)]
pub struct AkarCardSlots {
    pub header: u64,  // 0 if no header
    pub body: u64,
    pub footer: u64,  // 0 if no footer
}

#[repr(C)]
pub struct AkarLinkResult {
    pub clicked: bool,
    pub hovered: bool,
    pub pressed: bool,
}

#[repr(C)]
pub struct AkarButtonStyle {
    pub fill: u32,              // 0 = use default
    pub hover_fill: u32,        // 0 = use default
    pub pressed_fill: u32,      // 0 = use default
    pub border_color: u32,      // 0 = use default
    pub content_color: u32,     // 0 = use default
    pub text_style: AkarTextStyle,
}

#[repr(C)]
pub struct AkarBadgeStyle {
    pub fill: u32,
    pub border_color: u32,
    pub content_color: u32,
    pub text_style: AkarTextStyle,
}

#[repr(C)]
pub struct AkarSeparatorStyle {
    pub color: u32,        // 0 = use default
    pub thickness: f32,    // 0.0 = use default
}

#[repr(C)]
pub struct AkarStatStyle {
    pub title_color: u32,
    pub value_color: u32,
    pub description_color: u32,
    pub title_text_style: AkarTextStyle,
    pub value_text_style: AkarTextStyle,
    pub description_text_style: AkarTextStyle,
}

#[repr(C)]
pub struct AkarNavbarStyle {
    pub background: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub corner_radii: [f32; 4],
}

#[repr(C)]
pub struct AkarTabBarStyle {
    pub active_color: u32,
    pub inactive_color: u32,
    pub indicator_color: u32,
}
```

### Functions to add

For each component, add C functions. Null style pointer = defaults.

```rust
// Heading
pub unsafe extern "C" fn akar_heading(
    ctx: *mut AkarCtx, node_id: u64, text: *const c_char,
    level: u32, style: *const AkarTextStyle
)

// Paragraph
pub unsafe extern "C" fn akar_paragraph(
    ctx: *mut AkarCtx, node_id: u64, text: *const c_char,
    style: *const AkarTextStyle
)

// Link
pub unsafe extern "C" fn akar_link(
    ctx: *mut AkarCtx, node_id: u64, text: *const c_char,
    style: *const AkarTextStyle
) -> AkarLinkResult

// Card layout
pub unsafe extern "C" fn akar_card_layout(
    ctx: *mut AkarCtx, node_id: u64, options: *const AkarCardLayout
) -> AkarCardSlots

// Card paint
pub unsafe extern "C" fn akar_card(
    ctx: *mut AkarCtx, node_id: u64,
    slots: *const AkarCardSlots, style: *const AkarCardStyle
)

// Navbar layout
pub unsafe extern "C" fn akar_navbar_layout(
    ctx: *mut AkarCtx, node_id: u64
) -> AkarNavbarSlots

// Navbar paint
pub unsafe extern "C" fn akar_navbar_painted(
    ctx: *mut AkarCtx, node_id: u64, style: *const AkarNavbarStyle
)

// Button styled
pub unsafe extern "C" fn akar_button_styled(
    ctx: *mut AkarCtx, node_id: u64, text: *const c_char,
    variant: u32, style: *const AkarButtonStyle
) -> AkarButtonResult

// Badge styled
pub unsafe extern "C" fn akar_badge_styled(
    ctx: *mut AkarCtx, node_id: u64, text: *const c_char,
    variant: u32, style: *const AkarBadgeStyle
)

// Separator styled
pub unsafe extern "C" fn akar_separator_styled(
    ctx: *mut AkarCtx, node_id: u64, style: *const AkarSeparatorStyle
)

// Stat styled
pub unsafe extern "C" fn akar_stat_styled(
    ctx: *mut AkarCtx, node_id: u64,
    title: *const c_char, value: *const c_char,
    description: *const c_char, style: *const AkarStatStyle
)

// Tab bar styled
pub unsafe extern "C" fn akar_tab_bar_styled(
    ctx: *mut AkarCtx, node_id: u64,
    tabs: *const *const c_char, tab_count: u32,
    active_tab: u32, style: *const AkarTabBarStyle
) -> AkarTabBarResponse
```

### Helper functions

Add a helper to convert a nullable `*const AkarTextStyle` to `Option<akar_components::TextStyle>`:

```rust
fn c_text_style_to_rust(ptr: *const AkarTextStyle) -> Option<akar_components::TextStyle> {
    if ptr.is_null() { return None; }
    let s = unsafe { &*ptr };
    let mut style = akar_components::TextStyle::empty();
    let mut any = false;
    if s.font_size > 0.0 { style.font_size = Some(s.font_size); any = true; }
    // ... etc for each field using sentinel checks
    if any { Some(style) } else { None }
}
```

### Existing navbar C API

The current `akar_navbar` in the C API calls `akar_components::akar_navbar` which is now the old combined function. It should be updated to use `akar_navbar_layout` + paint separately, or keep using `akar_navbar_combined`. Check what's currently exported.

Actually, looking at lib.rs exports: `akar_navbar` is the paint-only function, `akar_navbar_combined` is the backward-compat one, and `akar_navbar_layout` is the constructor. The C API's `akar_navbar` should probably call `akar_navbar_combined` to maintain the same behavior.

### Tests

Add C integration tests in `crates/akar-c-api/tests/` if that directory exists, or add unit tests in the lib.rs. The tests should verify:
- Null style pointer produces default behavior
- Non-null style with overrides applies them
- Heading levels map correctly
- Card layout returns valid slot IDs

### Regenerate akar.h

After adding all C functions, regenerate `akar.h` with cbindgen:
```bash
cd crates/akar-c-api && cbindgen --config cbindgen.toml --crate akar-c-api --output ../../akar.h
```

Check if cbindgen.toml exists first. If not, check how akar.h was previously generated.

### Coding conventions
- Edition 2021, no emojis, no unnecessary comments
- All `unsafe` blocks should have clear safety invariants

After implementing, run `cargo test --workspace` and `cargo fmt` to verify.

---
**Output:**
Write your findings to exactly this path: /Users/brainless/Projects/akar/.pi-subagents/artifacts/outputs/873303e0/inline
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

## Acceptance Contract
Acceptance level: checked
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope

Required evidence: changed-files, tests-added, commands-run, residual-risks, no-staged-files

Finish with a fenced JSON block tagged `acceptance-report` in this shape:
Use empty arrays when no items apply; array fields contain strings unless object entries are shown.
`criteriaSatisfied[].status` must be exactly one of: satisfied, not-satisfied, not-applicable.
`commandsRun[].result` must be exactly one of: passed, failed, not-run.
`manualNotes` and `notes` are optional strings; an empty string means no note and does not satisfy `manual-notes` evidence.
```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "specific proof"
    }
  ],
  "changedFiles": [
    "src/file.ts"
  ],
  "testsAddedOrUpdated": [
    "test/file.test.ts"
  ],
  "commandsRun": [
    {
      "command": "command",
      "result": "passed",
      "summary": "short result"
    }
  ],
  "validationOutput": [
    "validation output or concise summary"
  ],
  "residualRisks": [
    "none"
  ],
  "noStagedFiles": true,
  "diffSummary": "short description of the diff",
  "reviewFindings": [
    "blocker: file.ts:12 - issue found, or no blockers"
  ],
  "manualNotes": "anything else the parent should know"
}
```