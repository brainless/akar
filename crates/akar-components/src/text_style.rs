#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontFamily {
    #[default]
    SansSerif,
    Serif,
    Monospace,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontWeight {
    #[default]
    Normal,
    Medium,
    Semibold,
    Bold,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextStyle {
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub color: Option<u32>,
    pub font_weight: Option<FontWeight>,
    pub font_family: Option<FontFamily>,
    pub align: Option<TextAlign>,
    pub wrap: Option<bool>,
}

impl TextStyle {
    pub fn empty() -> Self {
        Self {
            font_size: None,
            line_height: None,
            color: None,
            font_weight: None,
            font_family: None,
            align: None,
            wrap: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub(crate) struct ResolvedTextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub color: u32,
    pub font_weight: FontWeight,
    pub font_family: FontFamily,
    pub align: TextAlign,
    pub wrap: bool,
}

#[allow(dead_code)]
pub(crate) fn resolve_text_style(
    theme: &crate::AkarTheme,
    defaults: &ResolvedTextStyle,
    override_style: Option<&TextStyle>,
) -> ResolvedTextStyle {
    let _ = theme;

    let mut resolved = *defaults;

    if let Some(style) = override_style {
        if let Some(v) = style.font_size {
            resolved.font_size = v;
        }
        if let Some(v) = style.line_height {
            resolved.line_height = v;
        }
        if let Some(v) = style.color {
            resolved.color = v;
        }
        if let Some(v) = style.font_weight {
            resolved.font_weight = v;
        }
        if let Some(v) = style.font_family {
            resolved.font_family = v;
        }
        if let Some(v) = style.align {
            resolved.align = v;
        }
        if let Some(v) = style.wrap {
            resolved.wrap = v;
        }
    }

    resolved
}

#[allow(dead_code, clippy::needless_lifetimes)]
pub(crate) fn resolved_to_attrs<'a>(rt: &ResolvedTextStyle) -> glyphon::Attrs<'a> {
    let family = match rt.font_family {
        FontFamily::SansSerif => glyphon::Family::SansSerif,
        FontFamily::Serif => glyphon::Family::Serif,
        FontFamily::Monospace => glyphon::Family::Monospace,
    };
    let weight = match rt.font_weight {
        FontWeight::Normal => glyphon::Weight::NORMAL,
        FontWeight::Medium => glyphon::Weight::MEDIUM,
        FontWeight::Semibold => glyphon::Weight::SEMIBOLD,
        FontWeight::Bold => glyphon::Weight::BOLD,
    };
    glyphon::Attrs::new().family(family).weight(weight)
}

#[allow(dead_code)]
pub(crate) fn resolved_to_metrics(rt: &ResolvedTextStyle) -> glyphon::Metrics {
    glyphon::Metrics::new(rt.font_size, rt.line_height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AKAR_THEME_DARK, AKAR_THEME_LIGHT};

    fn h1_defaults(theme: &crate::AkarTheme) -> ResolvedTextStyle {
        ResolvedTextStyle {
            font_size: theme.font_size_heading_1,
            line_height: theme.font_size_heading_1 * 1.2,
            color: theme.base_content,
            font_weight: FontWeight::Bold,
            font_family: FontFamily::SansSerif,
            align: TextAlign::Start,
            wrap: false,
        }
    }

    #[test]
    fn theme_default_h1_uses_heading_1_size_and_bold() {
        let theme = AKAR_THEME_DARK;
        let resolved = resolve_text_style(&theme, &h1_defaults(&theme), None);

        assert_eq!(resolved.font_size, theme.font_size_heading_1);
        assert_eq!(resolved.font_weight, FontWeight::Bold);
        assert_eq!(resolved.align, TextAlign::Start);
        assert_eq!(resolved.color, theme.base_content);
    }

    #[test]
    fn partial_override_only_changes_listed_fields() {
        let theme = AKAR_THEME_LIGHT;
        let defaults = h1_defaults(&theme);
        let override_style = TextStyle {
            color: Some(0xff0000ff),
            ..TextStyle::empty()
        };

        let resolved = resolve_text_style(&theme, &defaults, Some(&override_style));

        assert_eq!(resolved.color, 0xff0000ff);
        assert_eq!(resolved.font_size, defaults.font_size);
        assert_eq!(resolved.font_weight, defaults.font_weight);
        assert_eq!(resolved.align, defaults.align);
    }

    #[test]
    fn full_override_replaces_every_field() {
        let theme = AKAR_THEME_DARK;
        let defaults = h1_defaults(&theme);
        let override_style = TextStyle {
            font_size: Some(48.0),
            line_height: Some(56.0),
            color: Some(0x00ff00ff),
            font_weight: Some(FontWeight::Semibold),
            font_family: Some(FontFamily::Serif),
            align: Some(TextAlign::Center),
            wrap: Some(true),
        };

        let resolved = resolve_text_style(&theme, &defaults, Some(&override_style));

        assert_eq!(resolved.font_size, 48.0);
        assert_eq!(resolved.line_height, 56.0);
        assert_eq!(resolved.color, 0x00ff00ff);
        assert_eq!(resolved.font_weight, FontWeight::Semibold);
        assert_eq!(resolved.font_family, FontFamily::Serif);
        assert_eq!(resolved.align, TextAlign::Center);
        assert!(resolved.wrap);
    }

    #[test]
    fn wrap_true_propagates_from_override() {
        let theme = AKAR_THEME_DARK;
        let defaults = ResolvedTextStyle {
            wrap: false,
            ..h1_defaults(&theme)
        };
        let override_style = TextStyle {
            wrap: Some(true),
            ..TextStyle::empty()
        };

        let resolved = resolve_text_style(&theme, &defaults, Some(&override_style));

        assert!(resolved.wrap);
    }

    #[test]
    fn center_alignment_maps_to_text_align_center() {
        let theme = AKAR_THEME_DARK;
        let defaults = ResolvedTextStyle {
            align: TextAlign::Start,
            ..h1_defaults(&theme)
        };
        let override_style = TextStyle {
            align: Some(TextAlign::Center),
            ..TextStyle::empty()
        };

        let resolved = resolve_text_style(&theme, &defaults, Some(&override_style));

        assert_eq!(resolved.align, TextAlign::Center);
    }

    #[test]
    fn resolved_to_attrs_maps_family_and_weight() {
        let rt = ResolvedTextStyle {
            font_size: 16.0,
            line_height: 20.0,
            color: 0xff,
            font_weight: FontWeight::Bold,
            font_family: FontFamily::Serif,
            align: TextAlign::Start,
            wrap: false,
        };

        let attrs = resolved_to_attrs(&rt);

        assert_eq!(attrs.weight, glyphon::Weight::BOLD);
        assert_eq!(attrs.family, glyphon::Family::Serif);
    }

    #[test]
    fn font_weight_enum_maps_to_glyphon_weights() {
        let sans = resolved_to_attrs(&ResolvedTextStyle {
            font_size: 16.0,
            line_height: 20.0,
            color: 0,
            font_weight: FontWeight::Normal,
            font_family: FontFamily::SansSerif,
            align: TextAlign::Start,
            wrap: false,
        });
        assert_eq!(sans.weight, glyphon::Weight::NORMAL);
        assert_eq!(sans.family, glyphon::Family::SansSerif);

        let mono = resolved_to_attrs(&ResolvedTextStyle {
            font_size: 16.0,
            line_height: 20.0,
            color: 0,
            font_weight: FontWeight::Medium,
            font_family: FontFamily::Monospace,
            align: TextAlign::Start,
            wrap: false,
        });
        assert_eq!(mono.weight, glyphon::Weight::MEDIUM);
        assert_eq!(mono.family, glyphon::Family::Monospace);

        let semi = resolved_to_attrs(&ResolvedTextStyle {
            font_size: 16.0,
            line_height: 20.0,
            color: 0,
            font_weight: FontWeight::Semibold,
            font_family: FontFamily::SansSerif,
            align: TextAlign::Start,
            wrap: false,
        });
        assert_eq!(semi.weight, glyphon::Weight::SEMIBOLD);
    }

    #[test]
    fn resolved_to_metrics_uses_font_size_and_line_height() {
        let rt = ResolvedTextStyle {
            font_size: 24.0,
            line_height: 30.0,
            color: 0,
            font_weight: FontWeight::Normal,
            font_family: FontFamily::SansSerif,
            align: TextAlign::Start,
            wrap: false,
        };

        let metrics = resolved_to_metrics(&rt);

        assert_eq!(metrics.font_size, 24.0);
        assert_eq!(metrics.line_height, 30.0);
    }

    #[test]
    fn theme_heading_sizes_match_spec() {
        let theme = AKAR_THEME_DARK;
        assert_eq!(theme.font_size_xl, 20.0);
        assert_eq!(theme.font_size_xxl, 24.0);
        assert_eq!(theme.font_size_heading_1, 36.0);
        assert_eq!(theme.font_size_heading_2, 30.0);
        assert_eq!(theme.font_size_heading_3, 24.0);
        assert_eq!(theme.font_size_heading_4, 20.0);

        let light = AKAR_THEME_LIGHT;
        assert_eq!(light.font_size_heading_1, 36.0);
        assert_eq!(light.muted_content, 0x71717aff);
    }

    #[test]
    fn theme_default_when_defaults_match_theme() {
        let theme = AKAR_THEME_DARK;
        let defaults = ResolvedTextStyle {
            font_size: theme.font_size_base,
            line_height: theme.font_size_base * 1.2,
            color: theme.base_content,
            font_weight: FontWeight::Normal,
            font_family: FontFamily::SansSerif,
            align: TextAlign::Start,
            wrap: false,
        };
        let resolved = resolve_text_style(&theme, &defaults, None);

        assert_eq!(resolved.font_size, theme.font_size_base);
        assert_eq!(resolved.color, theme.base_content);
    }

    #[test]
    fn cascade_order_defaults_then_override() {
        let theme = AKAR_THEME_DARK;
        let defaults = ResolvedTextStyle {
            font_size: theme.font_size_lg,
            line_height: 20.0,
            color: theme.base_content,
            font_weight: FontWeight::Normal,
            font_family: FontFamily::SansSerif,
            align: TextAlign::Start,
            wrap: false,
        };
        let override_style = TextStyle {
            font_size: Some(96.0),
            ..TextStyle::empty()
        };

        let resolved = resolve_text_style(&theme, &defaults, Some(&override_style));

        assert_eq!(resolved.font_size, 96.0);
        assert_eq!(resolved.line_height, 20.0);
    }

    #[test]
    fn defaults_with_no_override_propagates_defaults() {
        let theme = AKAR_THEME_DARK;
        let defaults = ResolvedTextStyle {
            font_size: theme.font_size_heading_2,
            line_height: theme.font_size_heading_2 * 1.2,
            color: theme.muted_content,
            font_weight: FontWeight::Semibold,
            font_family: FontFamily::Serif,
            align: TextAlign::Start,
            wrap: false,
        };

        let resolved = resolve_text_style(&theme, &defaults, None);

        assert_eq!(resolved.font_size, theme.font_size_heading_2);
        assert_eq!(resolved.line_height, theme.font_size_heading_2 * 1.2);
        assert_eq!(resolved.color, theme.muted_content);
        assert_eq!(resolved.font_weight, FontWeight::Semibold);
        assert_eq!(resolved.font_family, FontFamily::Serif);
    }
}
