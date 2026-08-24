use crate::catalog;

#[derive(Debug, Clone, PartialEq)]
pub enum RunMode {
    Run,
    Help,
    ListComponents,
    ListVariants(String),
    CaptureAll(String),
}

#[derive(Debug, Clone)]
pub struct CliConfig {
    pub mode: RunMode,
    pub component: Option<String>,
    #[allow(dead_code)] // used by variant showcase rendering in Task 2+
    pub variant: Option<String>,
    pub script: Option<String>,
    pub screenshot: Option<String>,
    pub dump_layout: bool,
    pub dump_frame: Option<String>,
    pub delay: f32,
    pub rtl: bool,
    pub exit: bool,
}

pub const HELP_TEXT: &str = r#"Usage: demo-rust [OPTIONS]

Options:
  --component <name>       Isolate a single component by canonical name or alias
  --variant <name>         Render a specific variant (requires --component)
  --screenshot <path>      Capture screenshot to path, then exit
  --script <path>          Run input script from path
  --dump-layout            Print layout node positions and exit
  --dump-frame <path>      Dump frame data as JSON to path
  --delay <seconds>        Screenshot delay in seconds (default: 5.0)
  --rtl                    Enable right-to-left layout
  --exit                   Exit after screenshot/script completes
  --list-components        List all available components and exit
  --list-variants <comp>   List variants for a component and exit
  --capture-all <dir>      Run full capture manifest to directory and exit
  --help                   Show this help message and exit

Discovery modes (--help, --list-components, --list-variants) cannot be combined
with rendering/capture options (--component, --variant, --screenshot, --script,
--dump-layout, --dump-frame, --delay, --rtl, --exit).

--variant requires --component. The component must have registered variants.

Aliases: tabs -> tab_bar, toast -> toasts"#;

pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<CliConfig, String> {
    let mut mode: Option<RunMode> = None;
    let mut component: Option<String> = None;
    let mut variant: Option<String> = None;
    let mut script: Option<String> = None;
    let mut screenshot: Option<String> = None;
    let mut dump_layout = false;
    let mut dump_frame: Option<String> = None;
    let mut delay: Option<f32> = None;
    let mut rtl = false;
    let mut exit = false;

    let mut arg_iter = args.into_iter();
    let _binary = arg_iter.next();

    while let Some(arg) = arg_iter.next() {
        match arg.as_str() {
            "--help" => {
                if mode.is_some() {
                    return Err(
                        "--help cannot be combined with other discovery or rendering options"
                            .into(),
                    );
                }
                mode = Some(RunMode::Help);
            }
            "--list-components" => {
                if mode.is_some() {
                    return Err(
                        "--list-components cannot be combined with other discovery or rendering options".into(),
                    );
                }
                mode = Some(RunMode::ListComponents);
            }
            "--list-variants" => {
                if mode.is_some() {
                    return Err(
                        "--list-variants cannot be combined with other discovery or rendering options"
                            .into(),
                    );
                }
                let comp = arg_iter
                    .next()
                    .ok_or("--list-variants requires a component name")?;
                mode = Some(RunMode::ListVariants(comp));
            }
            "--capture-all" => {
                if mode.is_some() {
                    return Err(
                        "--capture-all cannot be combined with other discovery or rendering options"
                            .into(),
                    );
                }
                let dir = arg_iter
                    .next()
                    .ok_or("--capture-all requires an output directory")?;
                mode = Some(RunMode::CaptureAll(dir));
            }
            "--component" => {
                if component.is_some() {
                    return Err("--component specified more than once".into());
                }
                let name = arg_iter
                    .next()
                    .ok_or("--component requires a component name")?;
                component = Some(name);
            }
            "--variant" => {
                if variant.is_some() {
                    return Err("--variant specified more than once".into());
                }
                let name = arg_iter.next().ok_or("--variant requires a variant name")?;
                variant = Some(name);
            }
            "--screenshot" => {
                let path = arg_iter.next().ok_or("--screenshot requires a file path")?;
                screenshot = Some(path);
            }
            "--script" => {
                let path = arg_iter.next().ok_or("--script requires a file path")?;
                script = Some(path);
            }
            "--dump-layout" => {
                dump_layout = true;
            }
            "--dump-frame" => {
                let path = arg_iter.next().ok_or("--dump-frame requires a file path")?;
                dump_frame = Some(path);
            }
            "--delay" => {
                let val = arg_iter.next().ok_or("--delay requires a numeric value")?;
                let parsed: f32 = val
                    .parse()
                    .map_err(|_| format!("invalid --delay value '{val}': expected a number"))?;
                if parsed < 0.0 {
                    return Err(format!("invalid --delay value '{val}': must be >= 0"));
                }
                delay = Some(parsed);
            }
            "--rtl" => {
                rtl = true;
            }
            "--exit" => {
                exit = true;
            }
            other => {
                return Err(format!("unknown flag '{other}'"));
            }
        }
    }

    let has_rendering = component.is_some()
        || variant.is_some()
        || script.is_some()
        || screenshot.is_some()
        || dump_layout
        || dump_frame.is_some()
        || delay.is_some()
        || rtl
        || exit;

    if let Some(ref m) = mode {
        if has_rendering {
            let name = match m {
                RunMode::Help => "--help",
                RunMode::ListComponents => "--list-components",
                RunMode::ListVariants(_) => "--list-variants",
                RunMode::CaptureAll(_) => "--capture-all",
                RunMode::Run => unreachable!(),
            };
            return Err(format!(
                "{name} cannot be combined with rendering or capture options"
            ));
        }
    }

    if variant.is_some() && component.is_none() {
        return Err("--variant requires --component".into());
    }

    if let Some(ref comp_name) = component {
        let entry = catalog::resolve(comp_name).ok_or_else(|| {
            let valid: Vec<&str> = catalog::names();
            format!(
                "unknown component '{comp_name}'. Valid components:\n  {}",
                valid.join(", ")
            )
        })?;

        if let Some(ref var_name) = variant {
            if entry.variants.is_empty() {
                return Err(format!(
                    "component '{}' has no registered variants",
                    entry.canonical_cli_name
                ));
            }
            if !entry.variants.contains(&var_name.as_str()) {
                return Err(format!(
                    "unknown variant '{}' for component '{}'. Valid variants:\n  {}",
                    var_name,
                    entry.canonical_cli_name,
                    entry.variants.join(", ")
                ));
            }
        }
    }

    if screenshot.is_some() && script.is_some() {
        return Err("--script and --screenshot are mutually exclusive".into());
    }

    let mode = mode.unwrap_or(RunMode::Run);

    Ok(CliConfig {
        mode,
        component,
        variant,
        script,
        screenshot,
        dump_layout,
        dump_frame,
        delay: delay.unwrap_or(5.0),
        rtl,
        exit,
    })
}

pub fn print_components() {
    for entry in catalog::CATALOG {
        println!("{}", entry.canonical_cli_name);
    }
}

pub fn print_variants(comp_name: &str) -> Result<(), String> {
    let entry = catalog::resolve(comp_name).ok_or_else(|| {
        let valid: Vec<&str> = catalog::names();
        format!(
            "unknown component '{comp_name}'. Valid components:\n  {}",
            valid.join(", ")
        )
    })?;
    if entry.variants.is_empty() {
        println!(
            "Component '{}' has no registered variants.",
            entry.canonical_cli_name
        );
    } else {
        for v in entry.variants {
            println!("{v}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_strs(args: &[&str]) -> Result<CliConfig, String> {
        parse(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn help_text_contains_usage() {
        assert!(HELP_TEXT.contains("Usage:"));
        assert!(HELP_TEXT.contains("--component"));
        assert!(HELP_TEXT.contains("--variant"));
        assert!(HELP_TEXT.contains("--list-components"));
        assert!(HELP_TEXT.contains("--list-variants"));
        assert!(HELP_TEXT.contains("--screenshot"));
        assert!(HELP_TEXT.contains("--script"));
        assert!(HELP_TEXT.contains("--dump-layout"));
        assert!(HELP_TEXT.contains("--dump-frame"));
        assert!(HELP_TEXT.contains("--delay"));
        assert!(HELP_TEXT.contains("--rtl"));
        assert!(HELP_TEXT.contains("--exit"));
    }

    #[test]
    fn default_values() {
        let cfg = parse_strs(&["demo-rust"]).unwrap();
        assert_eq!(cfg.mode, RunMode::Run);
        assert!(cfg.component.is_none());
        assert!(cfg.variant.is_none());
        assert!(cfg.script.is_none());
        assert!(cfg.screenshot.is_none());
        assert!(!cfg.dump_layout);
        assert!(cfg.dump_frame.is_none());
        assert_eq!(cfg.delay, 5.0);
        assert!(!cfg.rtl);
        assert!(!cfg.exit);
    }

    #[test]
    fn help_flag() {
        let cfg = parse_strs(&["demo-rust", "--help"]).unwrap();
        assert_eq!(cfg.mode, RunMode::Help);
    }

    #[test]
    fn list_components_flag() {
        let cfg = parse_strs(&["demo-rust", "--list-components"]).unwrap();
        assert_eq!(cfg.mode, RunMode::ListComponents);
    }

    #[test]
    fn list_variants_flag() {
        let cfg = parse_strs(&["demo-rust", "--list-variants", "button"]).unwrap();
        assert_eq!(cfg.mode, RunMode::ListVariants("button".into()));
    }

    #[test]
    fn every_canonical_component_parses() {
        for name in catalog::names() {
            let cfg = parse_strs(&["demo-rust", "--component", name]).unwrap();
            assert_eq!(cfg.component.as_deref(), Some(name));
        }
    }

    #[test]
    fn alias_tabs_resolves() {
        let cfg = parse_strs(&["demo-rust", "--component", "tabs"]).unwrap();
        assert_eq!(cfg.component.as_deref(), Some("tabs"));
    }

    #[test]
    fn alias_toast_resolves() {
        let cfg = parse_strs(&["demo-rust", "--component", "toast"]).unwrap();
        assert_eq!(cfg.component.as_deref(), Some("toast"));
    }

    #[test]
    fn every_valid_variant_parses() {
        for entry in catalog::CATALOG {
            for &v in entry.variants {
                let cfg = parse_strs(&[
                    "demo-rust",
                    "--component",
                    entry.canonical_cli_name,
                    "--variant",
                    v,
                ])
                .unwrap();
                assert_eq!(cfg.variant.as_deref(), Some(v));
            }
        }
    }

    #[test]
    fn variant_via_alias_parses() {
        let cfg = parse_strs(&["demo-rust", "--component", "tabs", "--variant", "boxed"]).unwrap();
        assert_eq!(cfg.variant.as_deref(), Some("boxed"));
    }

    #[test]
    fn list_variants_for_each_variant_bearing_component() {
        for entry in catalog::CATALOG {
            if entry.variants.is_empty() {
                continue;
            }
            let cfg =
                parse_strs(&["demo-rust", "--list-variants", entry.canonical_cli_name]).unwrap();
            assert_eq!(
                cfg.mode,
                RunMode::ListVariants(entry.canonical_cli_name.into())
            );
        }
    }

    #[test]
    fn list_variants_accepts_alias() {
        let cfg = parse_strs(&["demo-rust", "--list-variants", "tabs"]).unwrap();
        assert_eq!(cfg.mode, RunMode::ListVariants("tabs".into()));
    }

    #[test]
    fn unknown_component_errors() {
        let err = parse_strs(&["demo-rust", "--component", "nonexistent"]).unwrap_err();
        assert!(err.contains("unknown component"));
        assert!(err.contains("nonexistent"));
    }

    #[test]
    fn unknown_variant_errors() {
        let err =
            parse_strs(&["demo-rust", "--component", "button", "--variant", "nope"]).unwrap_err();
        assert!(err.contains("unknown variant"));
        assert!(err.contains("nope"));
        assert!(err.contains("solid"));
    }

    #[test]
    fn unknown_flag_errors() {
        let err = parse_strs(&["demo-rust", "--bogus"]).unwrap_err();
        assert!(err.contains("unknown flag"));
        assert!(err.contains("--bogus"));
    }

    #[test]
    fn missing_component_value_errors() {
        let err = parse_strs(&["demo-rust", "--component"]).unwrap_err();
        assert!(err.contains("requires a component name"));
    }

    #[test]
    fn missing_variant_value_errors() {
        let err = parse_strs(&["demo-rust", "--variant"]).unwrap_err();
        assert!(err.contains("requires a variant name"));
    }

    #[test]
    fn missing_screenshot_value_errors() {
        let err = parse_strs(&["demo-rust", "--screenshot"]).unwrap_err();
        assert!(err.contains("requires a file path"));
    }

    #[test]
    fn missing_script_value_errors() {
        let err = parse_strs(&["demo-rust", "--script"]).unwrap_err();
        assert!(err.contains("requires a file path"));
    }

    #[test]
    fn missing_delay_value_errors() {
        let err = parse_strs(&["demo-rust", "--delay"]).unwrap_err();
        assert!(err.contains("requires a numeric value"));
    }

    #[test]
    fn invalid_delay_value_errors() {
        let err = parse_strs(&["demo-rust", "--delay", "abc"]).unwrap_err();
        assert!(err.contains("invalid --delay"));
    }

    #[test]
    fn negative_delay_errors() {
        let err = parse_strs(&["demo-rust", "--delay", "-1.0"]).unwrap_err();
        assert!(err.contains("must be >= 0"));
    }

    #[test]
    fn duplicate_component_errors() {
        let err = parse_strs(&["demo-rust", "--component", "button", "--component", "badge"])
            .unwrap_err();
        assert!(err.contains("more than once"));
    }

    #[test]
    fn duplicate_variant_errors() {
        let err = parse_strs(&[
            "demo-rust",
            "--component",
            "button",
            "--variant",
            "solid",
            "--variant",
            "ghost",
        ])
        .unwrap_err();
        assert!(err.contains("more than once"));
    }

    #[test]
    fn variant_without_component_errors() {
        let err = parse_strs(&["demo-rust", "--variant", "solid"]).unwrap_err();
        assert!(err.contains("--variant requires --component"));
    }

    #[test]
    fn variant_on_non_variant_component_errors() {
        let err =
            parse_strs(&["demo-rust", "--component", "card", "--variant", "solid"]).unwrap_err();
        assert!(err.contains("no registered variants"));
    }

    #[test]
    fn script_screenshot_conflict() {
        let err = parse_strs(&[
            "demo-rust",
            "--screenshot",
            "/tmp/x.png",
            "--script",
            "/tmp/s.txt",
        ])
        .unwrap_err();
        assert!(err.contains("mutually exclusive"));
    }

    #[test]
    fn help_conflicts_with_rendering() {
        let err = parse_strs(&["demo-rust", "--help", "--component", "button"]).unwrap_err();
        assert!(err.contains("cannot be combined"));
    }

    #[test]
    fn list_components_conflicts_with_rendering() {
        let err = parse_strs(&[
            "demo-rust",
            "--list-components",
            "--screenshot",
            "/tmp/x.png",
        ])
        .unwrap_err();
        assert!(err.contains("cannot be combined"));
    }

    #[test]
    fn list_variants_conflicts_with_rendering() {
        let err = parse_strs(&["demo-rust", "--list-variants", "button", "--exit"]).unwrap_err();
        assert!(err.contains("cannot be combined"));
    }

    #[test]
    fn capture_all_flag() {
        let cfg = parse_strs(&["demo-rust", "--capture-all", "/tmp/out"]).unwrap();
        assert_eq!(cfg.mode, RunMode::CaptureAll("/tmp/out".into()));
    }

    #[test]
    fn capture_all_requires_dir() {
        let err = parse_strs(&["demo-rust", "--capture-all"]).unwrap_err();
        assert!(err.contains("requires an output directory"));
    }

    #[test]
    fn capture_all_conflicts_with_rendering() {
        let err = parse_strs(&["demo-rust", "--capture-all", "/tmp/out", "--exit"]).unwrap_err();
        assert!(err.contains("cannot be combined"));
    }

    #[test]
    fn rendering_options_parse_correctly() {
        let cfg = parse_strs(&[
            "demo-rust",
            "--component",
            "button",
            "--variant",
            "outline",
            "--screenshot",
            "/tmp/out.png",
            "--delay",
            "2.5",
            "--rtl",
            "--exit",
        ])
        .unwrap();
        assert_eq!(cfg.component.as_deref(), Some("button"));
        assert_eq!(cfg.variant.as_deref(), Some("outline"));
        assert_eq!(cfg.screenshot.as_deref(), Some("/tmp/out.png"));
        assert_eq!(cfg.delay, 2.5);
        assert!(cfg.rtl);
        assert!(cfg.exit);
    }

    #[test]
    fn dump_layout_and_dump_frame_parse() {
        let cfg = parse_strs(&[
            "demo-rust",
            "--dump-layout",
            "--dump-frame",
            "/tmp/frame.json",
        ])
        .unwrap();
        assert!(cfg.dump_layout);
        assert_eq!(cfg.dump_frame.as_deref(), Some("/tmp/frame.json"));
    }

    #[test]
    fn zero_delay_is_valid() {
        let cfg = parse_strs(&["demo-rust", "--delay", "0"]).unwrap();
        assert_eq!(cfg.delay, 0.0);
    }

    #[test]
    fn catalog_coverage_every_entry_has_canonical_name() {
        for entry in catalog::CATALOG {
            assert!(
                !entry.canonical_cli_name.is_empty(),
                "family '{}' has empty canonical_cli_name",
                entry.family
            );
        }
    }

    #[test]
    fn catalog_coverage_variant_bearing_have_at_least_one_variant() {
        for entry in catalog::CATALOG {
            if entry.family == "button"
                || entry.family == "badge"
                || entry.family == "alert"
                || entry.family == "toast"
                || entry.family == "tabs"
                || entry.family == "skeleton"
                || entry.family == "heading"
                || entry.family == "drawer"
                || entry.family == "tooltip"
                || entry.family == "text_input"
            {
                assert!(
                    !entry.variants.is_empty(),
                    "family '{}' expected to have variants but has none",
                    entry.family
                );
            }
        }
    }

    #[test]
    fn catalog_coverage_canonical_names_unique() {
        let names: Vec<&str> = catalog::CATALOG
            .iter()
            .map(|e| e.canonical_cli_name)
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(names.len(), sorted.len(), "canonical names are not unique");
    }

    #[test]
    fn catalog_coverage_variants_map_to_valid_entries() {
        for entry in catalog::CATALOG {
            for &v in entry.variants {
                assert!(!v.is_empty(), "empty variant in family '{}'", entry.family);
            }
        }
    }
}
