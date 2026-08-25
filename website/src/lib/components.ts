export interface ComponentEntry {
  family: string;
  name: string;
  category: string;
  description: string;
  cAbi: boolean;
  variants: string[];
  screenshot: string;
  isComposite: boolean;
}

export const implemented: ComponentEntry[] = [
  {
    family: "alert",
    name: "Alert",
    category: "Feedback",
    description:
      "Status messages with Info, Success, Warning, and Error color variants. Optional closable affordance.",
    cAbi: true,
    variants: ["info", "success", "warning", "error"],
    screenshot: "akar-alert.png",
    isComposite: false,
  },
  {
    family: "avatar",
    name: "Avatar",
    category: "Primitives",
    description:
      "Initials display in a circle with deterministic background color derived from the initials.",
    cAbi: true,
    variants: [],
    screenshot: "akar-avatar.png",
    isComposite: false,
  },
  {
    family: "badge",
    name: "Badge",
    category: "Primitives",
    description:
      "Status indicators and labels with Default, Primary, Success, Warning, Error, and Info variants.",
    cAbi: true,
    variants: ["default", "primary", "success", "warning", "error", "info"],
    screenshot: "akar-badge.png",
    isComposite: false,
  },
  {
    family: "button",
    name: "Button",
    category: "Inputs",
    description:
      "Primary action element with three visual variants: Solid (filled), Outline (bordered), and Ghost (minimal).",
    cAbi: true,
    variants: ["solid", "outline", "ghost"],
    screenshot: "akar-button.png",
    isComposite: false,
  },
  {
    family: "canvas",
    name: "Canvas",
    category: "Special",
    description:
      "Continuous LOD rendering with world-space transforms, group-level interaction, and portal mode for interactive detail.",
    cAbi: false,
    variants: [],
    screenshot: "akar-canvas.png",
    isComposite: false,
  },
  {
    family: "card",
    name: "Card",
    category: "Layout",
    description:
      "Container for grouping related content with background, border, and corner radius.",
    cAbi: true,
    variants: [],
    screenshot: "akar-card.png",
    isComposite: false,
  },
  {
    family: "checkbox",
    name: "Checkbox",
    category: "Inputs",
    description:
      "Binary toggle for multi-select options with checked, unchecked, hover, and pressed states.",
    cAbi: true,
    variants: [],
    screenshot: "akar-checkbox.png",
    isComposite: false,
  },
  {
    family: "container",
    name: "Container",
    category: "Layout",
    description:
      "Visual container that paints a caller-resolved rectangle with fill, border, corner radii, and optional shadow via BoxStyle.",
    cAbi: true,
    variants: [],
    screenshot: "akar-container.png",
    isComposite: false,
  },
  {
    family: "data_item",
    name: "Data Item",
    category: "Layout",
    description:
      "Composable visual shell over caller-provided content with hover, press, and click state reporting.",
    cAbi: true,
    variants: [],
    screenshot: "akar-data-item.png",
    isComposite: false,
  },
  {
    family: "data_list",
    name: "Data List",
    category: "Layout",
    description:
      "Fixed-height virtualized list scope with stable-key contract, visible-range rendering, and caller-owned scroll state.",
    cAbi: true,
    variants: [],
    screenshot: "akar-data-list.png",
    isComposite: false,
  },
  {
    family: "drawer",
    name: "Drawer",
    category: "Overlay/Navigation",
    description:
      "Slide-in panel from screen edges with Left and Right edge variants.",
    cAbi: true,
    variants: ["left", "right"],
    screenshot: "akar-drawer.png",
    isComposite: false,
  },
  {
    family: "dropdown",
    name: "Dropdown",
    category: "Overlay/Navigation",
    description:
      "Context menu or action list triggered by a button or interaction.",
    cAbi: true,
    variants: [],
    screenshot: "akar-dropdown.png",
    isComposite: false,
  },
  {
    family: "heading",
    name: "Heading",
    category: "Typography",
    description:
      "Section headings with H1 through H4 hierarchy levels and consistent styling.",
    cAbi: true,
    variants: ["h1", "h2", "h3", "h4"],
    screenshot: "akar-heading.png",
    isComposite: false,
  },
  {
    family: "label",
    name: "Label",
    category: "Typography",
    description: "Text labels for form fields and UI elements.",
    cAbi: true,
    variants: [],
    screenshot: "akar-label.png",
    isComposite: false,
  },
  {
    family: "link",
    name: "Link",
    category: "Typography",
    description:
      "Navigation links with hover states and external link support.",
    cAbi: true,
    variants: [],
    screenshot: "akar-link.png",
    isComposite: false,
  },
  {
    family: "modal",
    name: "Modal",
    category: "Overlay/Navigation",
    description: "Dialog overlay for focused interactions with backdrop scrim.",
    cAbi: true,
    variants: [],
    screenshot: "akar-modal.png",
    isComposite: false,
  },
  {
    family: "navbar",
    name: "Navbar",
    category: "Overlay/Navigation",
    description: "Top navigation bar with responsive menu and brand slot.",
    cAbi: true,
    variants: [],
    screenshot: "akar-navbar.png",
    isComposite: false,
  },
  {
    family: "paragraph",
    name: "Paragraph",
    category: "Typography",
    description: "Body text with proper line height and spacing.",
    cAbi: true,
    variants: [],
    screenshot: "akar-paragraph.png",
    isComposite: false,
  },
  {
    family: "progress",
    name: "Progress",
    category: "Feedback",
    description:
      "Determinate progress indicator showing completion percentage.",
    cAbi: true,
    variants: [],
    screenshot: "akar-progress.png",
    isComposite: false,
  },
  {
    family: "radio",
    name: "Radio",
    category: "Inputs",
    description: "Single-select from a group of options with mutual exclusion.",
    cAbi: true,
    variants: [],
    screenshot: "akar-radio.png",
    isComposite: false,
  },
  {
    family: "scroll_area",
    name: "Scroll Area",
    category: "Layout",
    description:
      "Scrolling and clipping scope that owns scroll state and clips overflow content to its bounds.",
    cAbi: true,
    variants: [],
    screenshot: "akar-scroll-area.png",
    isComposite: false,
  },
  {
    family: "select",
    name: "Select",
    category: "Inputs",
    description:
      "Dropdown selection from a list of options with open/closed states.",
    cAbi: true,
    variants: [],
    screenshot: "akar-select.png",
    isComposite: false,
  },
  {
    family: "separator",
    name: "Separator",
    category: "Layout",
    description: "Visual dividers between content sections.",
    cAbi: true,
    variants: [],
    screenshot: "akar-separator.png",
    isComposite: false,
  },
  {
    family: "skeleton",
    name: "Skeleton",
    category: "Feedback",
    description:
      "Loading placeholder content with Text, Card, and Circle shape variants.",
    cAbi: true,
    variants: ["text", "card", "circle"],
    screenshot: "akar-skeleton.png",
    isComposite: false,
  },
  {
    family: "slider",
    name: "Slider",
    category: "Inputs",
    description:
      "Range input for numeric values within bounds with drag interaction.",
    cAbi: true,
    variants: [],
    screenshot: "akar-slider.png",
    isComposite: false,
  },
  {
    family: "stat",
    name: "Stat",
    category: "Feedback",
    description:
      "Title, value, and description display for key metrics and statistics.",
    cAbi: true,
    variants: [],
    screenshot: "akar-stat.png",
    isComposite: false,
  },
  {
    family: "steps",
    name: "Steps",
    category: "Feedback",
    description:
      "Sequential progress indicator showing current step in a multi-step flow.",
    cAbi: true,
    variants: [],
    screenshot: "akar-steps.png",
    isComposite: false,
  },
  {
    family: "switch",
    name: "Switch",
    category: "Inputs",
    description:
      "Toggle between on and off states with on/off and hover visual states.",
    cAbi: true,
    variants: [],
    screenshot: "akar-switch.png",
    isComposite: false,
  },
  {
    family: "tabs",
    name: "Tab Bar",
    category: "Overlay/Navigation",
    description:
      "Navigation between content panels with Boxed, Lifted, Pills, and Underline style variants.",
    cAbi: true,
    variants: ["boxed", "lifted", "pills", "underline"],
    screenshot: "akar-tabs.png",
    isComposite: false,
  },
  {
    family: "text_input",
    name: "Text Input",
    category: "Inputs",
    description:
      "Single-line text input with Normal and Masked (password) variants, focus handling, selection, and clipboard support.",
    cAbi: true,
    variants: ["normal", "masked"],
    screenshot: "akar-text-input.png",
    isComposite: false,
  },
  {
    family: "textarea",
    name: "Textarea",
    category: "Inputs",
    description:
      "Multi-line text input for longer content with focus and edit states.",
    cAbi: true,
    variants: [],
    screenshot: "akar-textarea.png",
    isComposite: false,
  },
  {
    family: "toast",
    name: "Toast",
    category: "Feedback",
    description:
      "Non-blocking notifications with Info, Success, Warning, and Error variants. Supports click-to-dismiss; dismiss lifecycle is caller-owned.",
    cAbi: true,
    variants: ["info", "success", "warning", "error"],
    screenshot: "akar-toast.png",
    isComposite: false,
  },
  {
    family: "tooltip",
    name: "Tooltip",
    category: "Overlay/Navigation",
    description:
      "Contextual information on hover with Top, Bottom, Left, and Right placement variants.",
    cAbi: true,
    variants: ["top", "bottom", "left", "right"],
    screenshot: "akar-tooltip.png",
    isComposite: false,
  },
];

export const categories = [
  "Primitives",
  "Inputs",
  "Feedback",
  "Layout",
  "Overlay/Navigation",
  "Typography",
  "Special",
] as const;

export type Category = (typeof categories)[number];

export function implementedByCategory(category: Category): ComponentEntry[] {
  return implemented.filter((c) => c.category === category);
}
