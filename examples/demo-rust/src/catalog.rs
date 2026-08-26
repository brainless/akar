#[allow(dead_code)]
pub struct CatalogEntry {
    pub family: &'static str,
    pub canonical_cli_name: &'static str,
    pub aliases: &'static [&'static str],
    pub artifact_stem: &'static str,
    pub variants: &'static [&'static str],
    /// Registered `--state` values for this component, beyond the implicit
    /// "default" that every component accepts (a no-op reset). CLI parsing
    /// and the capture manifest share this list so an unknown or
    /// wrong-component state is rejected before any GPU/window creation.
    pub states: &'static [&'static str],
    pub c_abi: bool,
    pub website_category: &'static str,
    pub is_composite: bool,
}

#[allow(dead_code)]
pub static CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        family: "alert",
        canonical_cli_name: "alert",
        aliases: &[],
        artifact_stem: "alert",
        variants: &["info", "success", "warning", "error"],
        states: &["closable"],
        c_abi: true,
        website_category: "Feedback",
        is_composite: false,
    },
    CatalogEntry {
        family: "avatar",
        canonical_cli_name: "avatar",
        aliases: &[],
        artifact_stem: "avatar",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Primitives",
        is_composite: false,
    },
    CatalogEntry {
        family: "badge",
        canonical_cli_name: "badge",
        aliases: &[],
        artifact_stem: "badge",
        variants: &["default", "primary", "success", "warning", "error", "info"],
        states: &[],
        c_abi: true,
        website_category: "Primitives",
        is_composite: false,
    },
    CatalogEntry {
        family: "button",
        canonical_cli_name: "button",
        aliases: &[],
        artifact_stem: "button",
        variants: &["solid", "outline", "ghost"],
        states: &["outline-hover", "outline-pressed"],
        c_abi: true,
        website_category: "Inputs",
        is_composite: false,
    },
    CatalogEntry {
        family: "canvas",
        canonical_cli_name: "canvas",
        aliases: &[],
        artifact_stem: "canvas",
        variants: &[],
        states: &[],
        c_abi: false,
        website_category: "Special",
        is_composite: false,
    },
    CatalogEntry {
        family: "card",
        canonical_cli_name: "card",
        aliases: &[],
        artifact_stem: "card",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Layout",
        is_composite: false,
    },
    CatalogEntry {
        family: "checkbox",
        canonical_cli_name: "checkbox",
        aliases: &[],
        artifact_stem: "checkbox",
        variants: &[],
        states: &["checked", "unchecked-hover", "unchecked-pressed"],
        c_abi: true,
        website_category: "Inputs",
        is_composite: false,
    },
    CatalogEntry {
        family: "container",
        canonical_cli_name: "container",
        aliases: &[],
        artifact_stem: "container",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Layout",
        is_composite: false,
    },
    CatalogEntry {
        family: "data_item",
        canonical_cli_name: "data_item",
        aliases: &[],
        artifact_stem: "data_item",
        variants: &[],
        states: &["hovered", "pressed"],
        c_abi: true,
        website_category: "Layout",
        is_composite: false,
    },
    CatalogEntry {
        family: "data_list",
        canonical_cli_name: "data_list",
        aliases: &[],
        artifact_stem: "data_list",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Layout",
        is_composite: false,
    },
    CatalogEntry {
        family: "drawer",
        canonical_cli_name: "drawer",
        aliases: &[],
        artifact_stem: "drawer",
        variants: &["left", "right"],
        states: &[],
        c_abi: true,
        website_category: "Overlay/Navigation",
        is_composite: false,
    },
    CatalogEntry {
        family: "dropdown",
        canonical_cli_name: "dropdown",
        aliases: &[],
        artifact_stem: "dropdown",
        variants: &[],
        states: &["composite"],
        c_abi: true,
        website_category: "Overlay/Navigation",
        is_composite: false,
    },
    CatalogEntry {
        family: "heading",
        canonical_cli_name: "heading",
        aliases: &[],
        artifact_stem: "heading",
        variants: &["h1", "h2", "h3", "h4"],
        states: &[],
        c_abi: true,
        website_category: "Typography",
        is_composite: false,
    },
    CatalogEntry {
        family: "label",
        canonical_cli_name: "label",
        aliases: &[],
        artifact_stem: "label",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Typography",
        is_composite: false,
    },
    CatalogEntry {
        family: "link",
        canonical_cli_name: "link",
        aliases: &[],
        artifact_stem: "link",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Typography",
        is_composite: false,
    },
    CatalogEntry {
        family: "modal",
        canonical_cli_name: "modal",
        aliases: &[],
        artifact_stem: "modal",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Overlay/Navigation",
        is_composite: false,
    },
    CatalogEntry {
        family: "navbar",
        canonical_cli_name: "navbar",
        aliases: &[],
        artifact_stem: "navbar",
        variants: &[],
        states: &["composite"],
        c_abi: true,
        website_category: "Overlay/Navigation",
        is_composite: false,
    },
    CatalogEntry {
        family: "paragraph",
        canonical_cli_name: "paragraph",
        aliases: &[],
        artifact_stem: "paragraph",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Typography",
        is_composite: false,
    },
    CatalogEntry {
        family: "progress",
        canonical_cli_name: "progress",
        aliases: &[],
        artifact_stem: "progress",
        variants: &[],
        states: &["100%"],
        c_abi: true,
        website_category: "Feedback",
        is_composite: false,
    },
    CatalogEntry {
        family: "radio",
        canonical_cli_name: "radio",
        aliases: &[],
        artifact_stem: "radio",
        variants: &[],
        states: &["second-selected"],
        c_abi: true,
        website_category: "Inputs",
        is_composite: false,
    },
    CatalogEntry {
        family: "scroll_area",
        canonical_cli_name: "scroll_area",
        aliases: &[],
        artifact_stem: "scroll_area",
        variants: &[],
        states: &["scrolled"],
        c_abi: true,
        website_category: "Layout",
        is_composite: false,
    },
    CatalogEntry {
        family: "select",
        canonical_cli_name: "select",
        aliases: &[],
        artifact_stem: "select",
        variants: &[],
        states: &["open"],
        c_abi: true,
        website_category: "Inputs",
        is_composite: false,
    },
    CatalogEntry {
        family: "separator",
        canonical_cli_name: "separator",
        aliases: &[],
        artifact_stem: "separator",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Layout",
        is_composite: false,
    },
    CatalogEntry {
        family: "skeleton",
        canonical_cli_name: "skeleton",
        aliases: &[],
        artifact_stem: "skeleton",
        variants: &["text", "card", "circle"],
        states: &[],
        c_abi: true,
        website_category: "Feedback",
        is_composite: false,
    },
    CatalogEntry {
        family: "slider",
        canonical_cli_name: "slider",
        aliases: &[],
        artifact_stem: "slider",
        variants: &[],
        states: &["80%"],
        c_abi: true,
        website_category: "Inputs",
        is_composite: false,
    },
    CatalogEntry {
        family: "stat",
        canonical_cli_name: "stat",
        aliases: &[],
        artifact_stem: "stat",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Feedback",
        is_composite: false,
    },
    CatalogEntry {
        family: "steps",
        canonical_cli_name: "steps",
        aliases: &[],
        artifact_stem: "steps",
        variants: &[],
        states: &["step4"],
        c_abi: true,
        website_category: "Feedback",
        is_composite: false,
    },
    CatalogEntry {
        family: "switch",
        canonical_cli_name: "switch",
        aliases: &[],
        artifact_stem: "switch",
        variants: &[],
        states: &["on", "off-hover", "off-pressed"],
        c_abi: true,
        website_category: "Inputs",
        is_composite: false,
    },
    CatalogEntry {
        family: "tabs",
        canonical_cli_name: "tab_bar",
        aliases: &["tabs"],
        artifact_stem: "tabs",
        variants: &["boxed", "lifted", "pills", "underline"],
        states: &[],
        c_abi: true,
        website_category: "Overlay/Navigation",
        is_composite: false,
    },
    CatalogEntry {
        family: "text_input",
        canonical_cli_name: "text_input",
        aliases: &[],
        artifact_stem: "text_input",
        variants: &["normal", "masked"],
        states: &["normal-empty", "normal-focused", "masked-focused"],
        c_abi: true,
        website_category: "Inputs",
        is_composite: false,
    },
    CatalogEntry {
        family: "textarea",
        canonical_cli_name: "textarea",
        aliases: &[],
        artifact_stem: "textarea",
        variants: &[],
        states: &["focused"],
        c_abi: true,
        website_category: "Inputs",
        is_composite: false,
    },
    CatalogEntry {
        family: "toast",
        canonical_cli_name: "toasts",
        aliases: &["toast"],
        artifact_stem: "toast",
        variants: &["info", "success", "warning", "error"],
        states: &[],
        c_abi: true,
        website_category: "Feedback",
        is_composite: false,
    },
    CatalogEntry {
        family: "tooltip",
        canonical_cli_name: "tooltip",
        aliases: &[],
        artifact_stem: "tooltip",
        variants: &["top", "bottom", "left", "right"],
        states: &["visible", "hidden-trigger"],
        c_abi: true,
        website_category: "Overlay/Navigation",
        is_composite: false,
    },
    CatalogEntry {
        family: "list",
        canonical_cli_name: "list",
        aliases: &[],
        artifact_stem: "list",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Layout",
        is_composite: true,
    },
    CatalogEntry {
        family: "stats",
        canonical_cli_name: "stats",
        aliases: &[],
        artifact_stem: "stats",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Feedback",
        is_composite: true,
    },
    CatalogEntry {
        family: "form",
        canonical_cli_name: "form",
        aliases: &[],
        artifact_stem: "form",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Inputs",
        is_composite: true,
    },
    CatalogEntry {
        family: "i18n",
        canonical_cli_name: "i18n",
        aliases: &[],
        artifact_stem: "i18n",
        variants: &[],
        states: &[],
        c_abi: true,
        website_category: "Special",
        is_composite: true,
    },
];

#[allow(dead_code)]
pub fn by_canonical_name(name: &str) -> Option<&'static CatalogEntry> {
    CATALOG.iter().find(|e| e.canonical_cli_name == name)
}

#[allow(dead_code)]
pub fn by_alias(name: &str) -> Option<&'static CatalogEntry> {
    CATALOG.iter().find(|e| e.aliases.contains(&name))
}

#[allow(dead_code)]
pub fn resolve(name: &str) -> Option<&'static CatalogEntry> {
    by_canonical_name(name).or_else(|| by_alias(name))
}

#[allow(dead_code)]
pub fn names() -> Vec<&'static str> {
    CATALOG.iter().map(|e| e.canonical_cli_name).collect()
}

#[allow(dead_code)]
pub fn standalone_names() -> Vec<&'static str> {
    CATALOG
        .iter()
        .filter(|e| !e.is_composite)
        .map(|e| e.canonical_cli_name)
        .collect()
}

#[allow(dead_code)]
pub fn variants_for(name: &str) -> &'static [&'static str] {
    resolve(name).map(|e| e.variants).unwrap_or(&[])
}

#[allow(dead_code)]
pub fn states_for(name: &str) -> &'static [&'static str] {
    resolve(name).map(|e| e.states).unwrap_or(&[])
}

/// "default" is always valid: it means "no explicit state", the implicit
/// reset every component already supports. Any other value must be
/// registered against the resolved component's `states` list.
#[allow(dead_code)]
pub fn is_valid_state(entry: &CatalogEntry, state: &str) -> bool {
    state == "default" || entry.states.contains(&state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_37_entries() {
        assert_eq!(CATALOG.len(), 37);
    }

    #[test]
    fn standalone_count_is_33() {
        assert_eq!(standalone_names().len(), 33);
    }

    #[test]
    fn canonical_names_are_unique() {
        let mut names: Vec<&str> = CATALOG.iter().map(|e| e.canonical_cli_name).collect();
        let len_before = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), len_before);
    }

    #[test]
    fn aliases_do_not_duplicate_canonical() {
        for entry in CATALOG {
            for alias in entry.aliases {
                assert!(
                    by_canonical_name(alias).is_none(),
                    "Alias '{alias}' collides with a canonical name"
                );
            }
        }
    }

    #[test]
    fn resolve_finds_canonical() {
        let entry = resolve("button").unwrap();
        assert_eq!(entry.family, "button");
    }

    #[test]
    fn resolve_finds_alias() {
        let entry = resolve("tabs").unwrap();
        assert_eq!(entry.canonical_cli_name, "tab_bar");
    }

    #[test]
    fn resolve_finds_toast_alias() {
        let entry = resolve("toast").unwrap();
        assert_eq!(entry.canonical_cli_name, "toasts");
    }

    #[test]
    fn variants_for_unknown_returns_empty() {
        assert!(variants_for("nonexistent").is_empty());
    }

    #[test]
    fn variants_for_button() {
        assert_eq!(variants_for("button"), &["solid", "outline", "ghost"]);
    }

    #[test]
    fn states_for_unknown_returns_empty() {
        assert!(states_for("nonexistent").is_empty());
    }

    #[test]
    fn states_for_button() {
        assert_eq!(states_for("button"), &["outline-hover", "outline-pressed"]);
    }

    #[test]
    fn states_for_alert_via_alias_free_name() {
        assert_eq!(states_for("alert"), &["closable"]);
    }

    #[test]
    fn default_state_always_valid() {
        for entry in CATALOG {
            assert!(is_valid_state(entry, "default"));
        }
    }

    #[test]
    fn registered_states_are_valid() {
        for entry in CATALOG {
            for &s in entry.states {
                assert!(is_valid_state(entry, s));
            }
        }
    }

    #[test]
    fn unregistered_state_is_invalid() {
        let entry = by_canonical_name("card").unwrap();
        assert!(entry.states.is_empty());
        assert!(!is_valid_state(entry, "hovered"));
    }

    #[test]
    fn state_registered_on_one_component_is_invalid_on_another() {
        let card = by_canonical_name("card").unwrap();
        assert!(!is_valid_state(card, "closable"));
        let alert = by_canonical_name("alert").unwrap();
        assert!(!is_valid_state(alert, "outline-hover"));
    }

    #[test]
    fn canvas_has_no_c_abi() {
        let entry = by_canonical_name("canvas").unwrap();
        assert!(!entry.c_abi);
    }

    #[test]
    fn composites_are_marked() {
        for name in &["list", "stats", "form", "i18n"] {
            let entry = by_canonical_name(name).unwrap();
            assert!(entry.is_composite, "{name} should be composite");
        }
    }

    #[test]
    fn all_standalone_not_composite() {
        for entry in CATALOG {
            if !entry.is_composite {
                assert!(
                    !["list", "stats", "form", "i18n"].contains(&entry.canonical_cli_name),
                    "{} should not be in composite list",
                    entry.canonical_cli_name
                );
            }
        }
    }
}
