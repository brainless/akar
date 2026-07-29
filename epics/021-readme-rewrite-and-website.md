# Epic 021: README Rewrite and Project Website

**Status:** Draft
**Goal:** Rewrite README.md as a semi-technical introduction that showcases the project's current state and vision, and ship a project website based on Astro + Tailwind (astro-haze theme) with homepage, components page, and blog. Both README and website share screenshots and cross-link to each other.

**Prerequisite:** Epic 020 is `Status: Done`.

---

## Introduction

akar has 20 completed epics and a growing component catalog, but the README still reads like a technical reference manual. New visitors — developers and agents evaluating UI frameworks — need to see what akar looks like, understand the agent-built/agent-debuggable story, and know where the project is headed.

This epic has two deliverables:

1. **README.md rewrite** — a semi-technical introduction with component screenshots, quality commentary, the agent development story, and a clear pointer to the website for full documentation.
2. **Project website** — an Astro + Tailwind site (based on the astro-haze theme) with a homepage, a components catalog page, and a blog. Screenshots are shared between README and website. The website expands on the README with more screenshots, detailed component listings, and blog posts derived from completed epics.

### The agent story

akar is primarily built by coding agents — specifically MiMo v2.5 (vision + code). The development loop is: agent makes a change, captures a screenshot via the built-in debug toolchain, analyzes the visual result, and iterates. This epic itself was planned by an agent that read all 20 epics, captured fresh screenshots of every component, analyzed their quality, and wrote this spec. The website and README are the first public-facing artifacts of this workflow.

### Screenshot inventory (captured fresh for this epic)

| Screenshot | Source | Used in |
|---|---|---|
| Full demo (list tab) | `--screenshot` full window | README hero, website homepage |
| Form controls | `--component form` | README, website components page |
| Card | `--component card` | README, website components page |
| Heading | `--component heading` | website components page |
| Navbar | `--component navbar` | README, website components page |
| Drawer | `--component drawer` | README, website components page |
| Dropdown | `--component dropdown` | website components page |
| Stats | `--component stats` | README, website components page |
| Modal | `--component modal` | website components page |
| Toasts | `--component toasts` | website components page |
| List (virtualized) | `--component list` | README, website components page |
| Akar marketing page | `webpage-rust --site akar` | README, website homepage |

---

## Component quality assessment

Based on fresh screenshots captured on 2026-07-29:

### What looks good

- **Form controls** are the strongest area. Text inputs, textarea, checkbox, radio, switch, slider, and select all render cleanly with consistent styling. The select dropdown has proper chevron icons and clean option highlighting. The slider thumb is well-proportioned against its track.
- **Card** has clean header/body/footer separation with visible separators. The rounded corners and subtle border read well.
- **Navbar** is minimal and functional — logo, badge, menu items, and action buttons sit comfortably in a single row. The badge count overlay on the icon works.
- **Stats** show three cards with title, large number, and trend line. The layout is tight and readable. The steps progress bar and avatar row below demonstrate composition.
- **List** virtualization works — 10 items render with progress bars, and the scroll area clips correctly. Items have consistent row height and hover states.
- **The akar marketing page** (`webpage-rust --site akar`) is the most polished artifact. It has a proper navbar, centered hero with serif heading, stat cards, feature cards with monospace headings, a "Why akar" section, and a component showcase. This proves the component catalog can compose real webpage layouts.

### What needs improvement

- **Drawer** renders correctly (avatar, nav links with icons, proper background) but the left edge is clipped in the isolated screenshot — the drawer panel's left border is cut off. This is an auto-crop boundary issue, not a rendering bug. The drawer itself is solid.
- **Dropdown** menu items lack hover-state highlighting in the idle screenshot. The items render with consistent height and the active item (Option B) has a subtle tint, but hover/pressed states need visual differentiation.
- **Modal** is sparse — just a close button and placeholder text. The rounded corners and background overlay work, but real modal content (title, description, action buttons) is needed for the component to feel complete.
- **Toasts** render as a single info toast with a blue background. The toast needs dismiss-on-click feedback, variant styling (success, warning, error), and stack positioning for multiple toasts.
- **Heading** renders clean H1 text but the isolated screenshot shows only the text on a black background — the component needs its surrounding context (spacing, color) to be meaningful in isolation.
- **Dark theme** is the only theme exercised in screenshots. Light theme needs verification before the website ships.

---

## Tasks

### Task 1 — Rewrite README.md

Rewrite `README.md` as a semi-technical introduction. Structure:

1. **Title + tagline** — "akar: GPU-accelerated UI components for agents and developers"
2. **Full demo screenshot** — the `akar-full.png` screenshot showing the complete demo UI
3. **What is akar** — 2-3 paragraphs: immediate-mode, GPU-rendered, C ABI, language-neutral. Mention the component count (30+), the flexbox layout engine (taffy), and the rendering stack (wgpu + glyphon).
4. **Why akar** — the problem it solves: building desktop UI with wgpu today means writing a rect renderer, text pipeline, layout engine, and component primitives from scratch. akar collapses that.
5. **Built by agents, debuggable by agents** — the MiMo v2.5 story. Mention the screenshot toolchain, component isolation, scripted input injection, and the agent feedback loop. This is akar's differentiator.
6. **Component showcase** — a grid or row of component screenshots (form, card, navbar, drawer, stats, list) with brief labels. Use the shared screenshot paths.
7. **The akar marketing page** — screenshot of `webpage-rust --site akar` as proof that the component catalog composes real layouts.
8. **Quick start** — `cargo run --bin demo-rust` and the screenshot commands.
9. **Stack** — wgpu 29, glyphon, taffy, glam, cbindgen. Same table as current README.
10. **Status** — pre-alpha, link to `epics/` for roadmap.
11. **Documentation** — "Full documentation and component catalog at [akar.dev](https://akar.dev) (coming soon)."
12. **License** — MIT.
13. **Footer links** — GitHub: https://github.com/brainless/akar

Remove from current README: the detailed text editing/clipboard section (move to website docs), the detailed screenshot workflow section (summarize, link to website/DEVELOP.md), the "For whom" section (fold into "Why akar").

### Task 2 — Set up Astro project from astro-haze

Create `website/` directory at the project root. Initialize from the astro-haze theme:

- Copy the astro-haze source from `~/Projects/astro-haze/` into `website/`.
- Update `package.json` name to `akar-website`.
- Update `site.config.ts`:
  - `name`: "akar"
  - `title`: "akar - GPU-Accelerated UI Components"
  - `description`: "A GPU-accelerated, language-neutral UI component library for agents and developers"
  - `url`: "https://akar.dev"
  - `nav.main`: Home (`/`), Components (`/components/`), Blog (`/blog/`)
  - Remove Portfolio, Landing, About from nav.
  - `features.portfolio`: false, `features.landing`: false
  - `social.github`: "https://github.com/brainless/akar"
  - `footer.links`: only GitHub
- Configure Tailwind (astro-haze uses Tailwind — verify and set up).
- Copy shared screenshots into `website/public/screenshots/`.
- Verify `npm run dev` serves the homepage.

### Task 3 — Build homepage

Replace the astro-haze index page with the akar homepage. Structure:

1. **Hero** — "akar" heading, tagline ("GPU-accelerated UI components for agents and developers"), two CTA buttons: "View on GitHub" and "Components".
2. **Full demo screenshot** — the shared `akar-full.png` in a rounded-corner Card component (re-using astro-haze's Card UI).
3. **Features grid** — 3 or 4 cards: "30+ Components", "Language Neutral (C ABI)", "Immediate Mode", "Built by Agents". Each with a short description and optionally a small screenshot.
4. **Component preview** — a row of 3-4 component screenshots (form, card, navbar, stats) in Card wrappers with labels.
5. **The akar page** — screenshot of the marketing page as proof of real-world composition.
6. **"Docs coming soon"** — a note that full documentation is under development.
7. **Footer** — compact, only GitHub link.

Use the Card component from astro-haze's UI components for screenshot display. The glassmorphism style of astro-haze works well for a dark-themed GPU UI project.

### Task 4 — Build components page

Create `website/src/pages/components.astro`. This page lists all components akar has implemented and planned, with the goal of reaching parity with shadcn/ui and daisyUI.

Structure:

1. **Page heading** — "Components" with a subtitle: "30+ components ready to use. Targeting parity with shadcn/ui and daisyUI."
2. **Component grid** — each component in a Card with:
   - Component name
   - Screenshot (where available, using the shared `--component` screenshots)
   - Status badge: "Implemented" (green) or "Planned" (gray)
   - Brief description
3. **Implemented components** (from demo-rust `--list-components` plus Epic 020 additions):
   - Button, Badge, Label, Link, Separator, Spinner, Kbd (primitives)
   - Input, Textarea, Checkbox, Radio, Switch, Slider, Select (inputs)
   - Alert, Tooltip, Toast (feedback)
   - Card, Table, Tab Bar + Tab Panel, Scroll Area, Data List (layout)
   - Dialog/Modal, Drawer, Dropdown, Navbar (overlay/navigation)
   - Heading, Paragraph (typography)
   - Canvas (with LOD and portals)
   - Data Item (presentation primitive)
4. **Planned components** — reference shadcn/ui and daisyUI catalogs:
   - Accordion/Collapse, Breadcrumb, Pagination, Command Palette, Popover
   - Skeleton, Spinner, Avatar, Calendar, DatePicker
   - Toast stack, Snackbar, Bottom Sheet
   - Split Pane, Resizable panels
   - Combobox, Date Range Picker, File Input
5. **"Contributing" note** — link to the GitHub repo for contributions.

Cross-reference `~/Projects/shadcn_ui/` and `~/Projects/daisyui/` for the planned component list. The component names should follow akar's naming conventions (from Epic 001 Task 8 research).

### Task 5 — Build blog infrastructure

Create the blog section:

1. **Blog listing page** (`website/src/pages/blog/index.astro`):
   - No top hero or featured Card (per requirements).
   - Simple list of blog posts, each as a Card with:
     - Main screenshot as the card image
     - Post title
     - Date
     - Short excerpt
   - Sorted by date, newest first.

2. **Blog post layout** — adapt astro-haze's blog layout. Each post has:
   - Title, date, reading time
   - Main screenshot at the top
   - MDX content
   - No share buttons, no related posts (keep it simple)

3. **Content collection** — configure `src/content.config.ts` for blog posts with frontmatter: `title`, `date`, `description`, `image` (screenshot path), `tags`.

### Task 6 — Convert epics to blog posts

Assess which epics work as blog posts and create them:

**Blog post 1: "How akar is built by agents"**
- Source: Epic 001 (exploration/architecture), git log, general project story
- Content: The vision of agent-built UI frameworks, how MiMo v2.5 was used, the development loop (change → screenshot → analyze → iterate), the debug toolchain that makes it possible. Include the full demo screenshot and a few component screenshots.
- Screenshot: `akar-full.png` + a few component isolations

**Blog post 2: "Screenshot-driven development: akar's debug toolchain"**
- Source: Epics 013, 014, 015
- Content: How the screenshot utility evolved from basic capture to scripted input injection, component isolation, and visual diffing. Show the workflow: `--list-components`, `--component X --screenshot`, `--script`, `--dump-frame`, `akar-diff`. Include before/after diff screenshots if available.
- Screenshots: component isolation examples, diff output

**Blog post 3: "Building a webpage with akar components"**
- Source: Epics 019, 020
- Content: The journey from raw `push_quad`/`push_text` calls (MiMo page) to a fully component-based marketing page. Show the before/after: raw rendering vs. component APIs. Discuss the component lifecycle (construct/compute/paint), text measurement, and typography system.
- Screenshots: MiMo page (`--site mimo`), akar page (`--site akar`)

**Blog post 4: "Canvas LOD: from overview to interactive detail"**
- Source: Epic 016
- Content: How canvas provides continuous level of detail — objects show progressively richer representations as they zoom in, from dots to outlines to previews to full interactive portals with standard akar components.
- Screenshots: canvas-basic-rust examples at different LOD levels

**Blog post 5: "Text editing in immediate mode"**
- Source: Epic 018
- Content: How akar handles text selection, cursor movement, copy/paste, and clipboard interop in an immediate-mode framework. The `TextEditState` design, keybinding configuration, and the platform-neutral clipboard boundary.
- Screenshots: form component showing text inputs

Create the MDX files in `website/src/content/blog/` with frontmatter. Each post should be 400-800 words — technical but accessible.

### Task 7 — Copy screenshots to website

Copy the shared screenshots into `website/public/screenshots/`:

```
website/public/screenshots/
  akar-full.png          # full demo
  akar-form.png          # form controls
  akar-card.png          # card component
  akar-heading.png       # heading
  akar-navbar.png        # navbar
  akar-drawer.png        # drawer
  akar-dropdown.png      # dropdown
  akar-stats.png         # stats
  akar-modal.png         # modal
  akar-toasts.png        # toasts
  akar-list.png          # virtualized list
  akar-website.png       # akar marketing page
```

These are sourced from `/tmp/akar-*.png` captured during this epic. The README references these same files (via relative paths or GitHub raw URLs). The website references them via `/screenshots/`.

### Task 8 — Compact footer

Both README and website footer should be minimal:

**README footer:**
```markdown
## License

MIT

[GitHub](https://github.com/brainless/akar)
```

**Website footer:**
- Single line: "akar" on the left, GitHub icon/link on the right.
- No other links. No sitemap, RSS, privacy, terms links.
- Dark background, minimal height.

### Task 9 — Cross-link README and website

- README "Documentation" section links to `https://akar.dev`.
- Website homepage has a "Read the README" or "GitHub" link in the hero.
- Blog posts link back to the relevant epics in the repo.
- README component screenshots link to the website components page for full details.

### Task 10 — Verify and polish

- Run `npm run build` in `website/` to verify the site builds.
- Run `npm run check` for TypeScript validation.
- Verify all screenshot paths resolve correctly.
- Verify blog posts render with images.
- Verify the components page lists all implemented and planned components.
- Verify the footer is compact on all pages.
- Verify nav has only Home, Components, Blog.
- Run `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` to ensure no regressions.
- Take a final full-window screenshot of the website homepage for the README.

---

## Acceptance Criteria

- [ ] README.md is rewritten with screenshots, agent story, component showcase, and link to website.
- [ ] Website builds and serves with `npm run dev`.
- [ ] Homepage has hero, demo screenshot, features grid, component preview, and compact footer.
- [ ] Components page lists all implemented components with screenshots and status badges.
- [ ] Components page lists planned components targeting shadcn/ui and daisyUI parity.
- [ ] Blog has 5 posts derived from epics, each with a main screenshot.
- [ ] Blog listing page shows posts as Cards with images, no hero.
- [ ] Footer is compact on all pages — only GitHub link.
- [ ] Nav has only Home, Components, Blog.
- [ ] Screenshots are shared between README and website.
- [ ] README and website cross-link to each other.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace -- -D warnings` passes.
- [ ] `npm run build` succeeds in `website/`.

---

## Explicit Deferrals

- Full documentation site (API reference, guides) — "docs coming soon" placeholder only.
- Dark/light theme toggle on the website — ship with the theme astro-haze provides.
- Search functionality — astro-haze includes astro-pagefind but it can be configured later.
- RSS feed — astro-haze generates one, can be kept or removed.
- Responsive mobile layout — verify but do not block on perfect mobile experience.
- Perceptual diff in akar-diff — deferred to a future epic.
- Website deployment (CI/CD, hosting) — out of scope for this epic.
