export type CrosswalkStatus =
  "implemented_equivalent" | "planned" | "alias" | "excluded";

export interface CrosswalkEntry {
  name: string;
  source: "daisyui" | "shadcn";
  status: CrosswalkStatus;
  reason: string;
}

function daisy(
  name: string,
  status: CrosswalkStatus,
  reason: string,
): CrosswalkEntry {
  return { name, source: "daisyui", status, reason };
}

function shadcn(
  name: string,
  status: CrosswalkStatus,
  reason: string,
): CrosswalkEntry {
  return { name, source: "shadcn", status, reason };
}

export const crosswalk: CrosswalkEntry[] = [
  // DaisyUI components
  daisy(
    "alert",
    "implemented_equivalent",
    "akar alert with Info/Success/Warning/Error variants",
  ),
  daisy(
    "avatar",
    "implemented_equivalent",
    "akar avatar with initials and image support",
  ),
  daisy(
    "badge",
    "implemented_equivalent",
    "akar badge with six color variants",
  ),
  daisy(
    "breadcrumbs",
    "planned",
    "Hierarchical navigation trail; no akar equivalent yet",
  ),
  daisy(
    "button",
    "implemented_equivalent",
    "akar button with Solid/Outline/Ghost variants",
  ),
  daisy(
    "calendar",
    "planned",
    "Date picker with month view; no akar equivalent yet",
  ),
  daisy("card", "implemented_equivalent", "akar card container"),
  daisy(
    "carousel",
    "planned",
    "Image/content carousel; no akar equivalent yet",
  ),
  daisy(
    "chat",
    "excluded",
    "Chat bubble layout; application-specific composition, not a library primitive",
  ),
  daisy(
    "checkbox",
    "implemented_equivalent",
    "akar checkbox with checked/unchecked states",
  ),
  daisy(
    "collapse",
    "alias",
    "Alias for accordion/collapse; planned as a single akar family",
  ),
  daisy(
    "countdown",
    "excluded",
    "Numeric countdown display; application-specific, not a general UI primitive",
  ),
  daisy(
    "diff",
    "excluded",
    "Visual diff display; application-specific, not a general UI primitive",
  ),
  daisy(
    "divider",
    "implemented_equivalent",
    "akar separator serves the same role",
  ),
  daisy(
    "dock",
    "excluded",
    "Bottom navigation dock; mobile-specific pattern, not in akar scope",
  ),
  daisy(
    "drawer",
    "implemented_equivalent",
    "akar drawer with Left/Right edge variants",
  ),
  daisy("dropdown", "implemented_equivalent", "akar dropdown menu"),
  daisy(
    "fab",
    "excluded",
    "Floating action button; application-specific pattern, not a standalone component",
  ),
  daisy(
    "fieldset",
    "excluded",
    "Form field grouping; handled by composition, not a standalone akar component",
  ),
  daisy("fileinput", "planned", "File upload input; no akar equivalent yet"),
  daisy(
    "filter",
    "excluded",
    "Filter chip group; application-specific composition",
  ),
  daisy(
    "footer",
    "excluded",
    "Page footer layout; application composition, not a library component",
  ),
  daisy(
    "hero",
    "excluded",
    "Hero section layout; application composition, not a library component",
  ),
  daisy(
    "hover3d",
    "excluded",
    "3D hover effect; visual effect, not a structural component",
  ),
  daisy(
    "hovergallery",
    "excluded",
    "Hover image gallery; application-specific composition",
  ),
  daisy(
    "indicator",
    "planned",
    "Positioned badge/dot indicator; no akar equivalent yet",
  ),
  daisy(
    "input",
    "implemented_equivalent",
    "akar text_input with Normal/Masked variants",
  ),
  daisy(
    "kbd",
    "planned",
    "Keyboard shortcut display; no akar component family yet",
  ),
  daisy("label", "implemented_equivalent", "akar label for form fields"),
  daisy("link", "implemented_equivalent", "akar link with hover states"),
  daisy(
    "list",
    "implemented_equivalent",
    "akar data_list with virtualization; also data_item for list items",
  ),
  daisy(
    "loading",
    "alias",
    "Alias for spinner/loading; planned as a single akar family",
  ),
  daisy("mask", "planned", "Image mask/clip shapes; no akar equivalent yet"),
  daisy(
    "menu",
    "implemented_equivalent",
    "akar dropdown and navbar cover menu patterns",
  ),
  daisy(
    "mockup",
    "excluded",
    "Device mockup frames; presentation tool, not a UI component",
  ),
  daisy("modal", "implemented_equivalent", "akar modal dialog"),
  daisy("navbar", "implemented_equivalent", "akar navbar with responsive menu"),
  daisy("progress", "implemented_equivalent", "akar progress indicator"),
  daisy(
    "radialprogress",
    "planned",
    "Circular progress indicator; no akar equivalent yet",
  ),
  daisy(
    "radio",
    "implemented_equivalent",
    "akar radio with single-select group",
  ),
  daisy("range", "implemented_equivalent", "akar slider serves the same role"),
  daisy("rating", "planned", "Star rating input; no akar equivalent yet"),
  daisy("select", "implemented_equivalent", "akar select dropdown"),
  daisy(
    "skeleton",
    "implemented_equivalent",
    "akar skeleton with Text/Card/Circle variants",
  ),
  daisy(
    "stack",
    "excluded",
    "Stacked element layout; handled by flex layout, not a standalone component",
  ),
  daisy(
    "stat",
    "implemented_equivalent",
    "akar stat with title/value/description",
  ),
  daisy("status", "excluded", "Status dot; trivial visual, covered by badge"),
  daisy(
    "steps",
    "implemented_equivalent",
    "akar steps sequential progress indicator",
  ),
  daisy(
    "swap",
    "excluded",
    "Toggle swap animation; visual effect, not a structural component",
  ),
  daisy(
    "tab",
    "implemented_equivalent",
    "akar tab_bar with Boxed/Lifted/Pills/Underline variants",
  ),
  daisy(
    "table",
    "planned",
    "Structured data table; no akar component family yet",
  ),
  daisy("textarea", "implemented_equivalent", "akar textarea multi-line input"),
  daisy(
    "textrotate",
    "excluded",
    "Rotating text animation; visual effect, not a structural component",
  ),
  daisy(
    "timeline",
    "planned",
    "Vertical timeline display; no akar equivalent yet",
  ),
  daisy(
    "toast",
    "implemented_equivalent",
    "akar toast with Info/Success/Warning/Error variants",
  ),
  daisy("toggle", "implemented_equivalent", "akar switch serves the same role"),
  daisy(
    "tooltip",
    "implemented_equivalent",
    "akar tooltip with four placement variants",
  ),
  daisy(
    "validator",
    "excluded",
    "Form validation display; handled by composition, not a standalone component",
  ),

  // shadcn/ui components
  shadcn(
    "accordion",
    "planned",
    "Expandable content sections; no akar equivalent yet",
  ),
  shadcn("alert", "implemented_equivalent", "akar alert with color variants"),
  shadcn(
    "alert-dialog",
    "implemented_equivalent",
    "akar modal covers confirmation dialogs",
  ),
  shadcn(
    "aspect-ratio",
    "planned",
    "Aspect ratio container; no akar equivalent yet",
  ),
  shadcn(
    "avatar",
    "implemented_equivalent",
    "akar avatar with initials and image support",
  ),
  shadcn(
    "badge",
    "implemented_equivalent",
    "akar badge with six color variants",
  ),
  shadcn(
    "breadcrumb",
    "planned",
    "Hierarchical navigation trail; no akar equivalent yet",
  ),
  shadcn(
    "button",
    "implemented_equivalent",
    "akar button with Solid/Outline/Ghost variants",
  ),
  shadcn(
    "button-group",
    "excluded",
    "Button grouping layout; composition pattern, not a standalone component",
  ),
  shadcn(
    "calendar",
    "planned",
    "Date picker with month view; no akar equivalent yet",
  ),
  shadcn("card", "implemented_equivalent", "akar card container"),
  shadcn(
    "carousel",
    "planned",
    "Image/content carousel; no akar equivalent yet",
  ),
  shadcn(
    "chart",
    "excluded",
    "Chart wrapper; data visualization, not a UI primitive",
  ),
  shadcn(
    "checkbox",
    "implemented_equivalent",
    "akar checkbox with checked/unchecked states",
  ),
  shadcn(
    "collapsible",
    "alias",
    "Alias for accordion/collapse; planned as a single akar family",
  ),
  shadcn(
    "combobox",
    "planned",
    "Searchable dropdown with filtering; no akar equivalent yet",
  ),
  shadcn(
    "command",
    "planned",
    "Command palette interface; no akar equivalent yet",
  ),
  shadcn(
    "context-menu",
    "planned",
    "Right-click context menu; no akar equivalent yet",
  ),
  shadcn("dialog", "implemented_equivalent", "akar modal dialog"),
  shadcn(
    "drawer",
    "implemented_equivalent",
    "akar drawer with Left/Right edge variants",
  ),
  shadcn("dropdown-menu", "implemented_equivalent", "akar dropdown menu"),
  shadcn(
    "empty",
    "excluded",
    "Empty state placeholder; composition pattern, not a standalone component",
  ),
  shadcn(
    "field",
    "excluded",
    "Form field grouping; handled by composition with label + input",
  ),
  shadcn(
    "form",
    "excluded",
    "Form wrapper with validation; application composition, not a standalone component",
  ),
  shadcn("hover-card", "planned", "Rich hover popup; no akar equivalent yet"),
  shadcn(
    "input",
    "implemented_equivalent",
    "akar text_input with Normal/Masked variants",
  ),
  shadcn(
    "input-group",
    "excluded",
    "Input grouping layout; composition pattern, not a standalone component",
  ),
  shadcn("input-otp", "planned", "OTP code input; no akar equivalent yet"),
  shadcn(
    "item",
    "implemented_equivalent",
    "akar data_item covers list item patterns",
  ),
  shadcn("label", "implemented_equivalent", "akar label for form fields"),
  shadcn("menubar", "planned", "Application menu bar; no akar equivalent yet"),
  shadcn(
    "navigation-menu",
    "planned",
    "Navigation menu with submenus; no akar equivalent yet",
  ),
  shadcn(
    "pagination",
    "planned",
    "Page navigation for lists; no akar equivalent yet",
  ),
  shadcn(
    "popover",
    "planned",
    "Floating content triggered by interaction; no akar equivalent yet",
  ),
  shadcn("progress", "implemented_equivalent", "akar progress indicator"),
  shadcn(
    "radio-group",
    "implemented_equivalent",
    "akar radio with single-select group",
  ),
  shadcn(
    "resizable",
    "planned",
    "Resizable panel layout; no akar equivalent yet",
  ),
  shadcn(
    "scroll-area",
    "implemented_equivalent",
    "akar scroll_area with clipping and scroll ownership",
  ),
  shadcn("select", "implemented_equivalent", "akar select dropdown"),
  shadcn("separator", "implemented_equivalent", "akar separator divider"),
  shadcn(
    "sheet",
    "implemented_equivalent",
    "akar drawer covers slide-in panel patterns",
  ),
  shadcn(
    "sidebar",
    "planned",
    "Application sidebar navigation; no akar equivalent yet",
  ),
  shadcn(
    "skeleton",
    "implemented_equivalent",
    "akar skeleton with Text/Card/Circle variants",
  ),
  shadcn("slider", "implemented_equivalent", "akar slider range input"),
  shadcn(
    "sonner",
    "alias",
    "Toast notification library; akar toast covers the same pattern",
  ),
  shadcn("spinner", "planned", "Loading spinner; no akar component family yet"),
  shadcn("switch", "implemented_equivalent", "akar switch toggle"),
  shadcn(
    "table",
    "planned",
    "Structured data table; no akar component family yet",
  ),
  shadcn(
    "tabs",
    "implemented_equivalent",
    "akar tab_bar with four style variants",
  ),
  shadcn(
    "textarea",
    "implemented_equivalent",
    "akar textarea multi-line input",
  ),
  shadcn(
    "toast",
    "implemented_equivalent",
    "akar toast with Info/Success/Warning/Error variants",
  ),
  shadcn(
    "toggle",
    "implemented_equivalent",
    "akar switch serves the same role",
  ),
  shadcn(
    "toggle-group",
    "planned",
    "Group of toggle buttons; no akar equivalent yet",
  ),
  shadcn(
    "tooltip",
    "implemented_equivalent",
    "akar tooltip with four placement variants",
  ),
  shadcn(
    "kbd",
    "planned",
    "Keyboard shortcut display; no akar component family yet",
  ),
  shadcn(
    "native-select",
    "alias",
    "Native HTML select; akar select covers the same pattern",
  ),
  shadcn(
    "direction",
    "excluded",
    "RTL/LTR direction provider; layout utility, not a visual component",
  ),
];

export function plannedComponents(): CrosswalkEntry[] {
  const seen = new Set<string>();
  return crosswalk.filter((entry) => {
    if (entry.status !== "planned") return false;
    const key = entry.name.toLowerCase();
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}
