#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontFamily {
    #[default]
    SansSerif,
    Serif,
    Monospace,
    /// A family loaded at runtime, addressed by the handle returned from
    /// `TextPipeline::load_font_bytes` / `akar_load_font_bytes`.
    Named(u32),
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

/// Builds the owned font request handed to `TextPipeline::set_text_styled`.
///
/// The named-family handle is resolved inside `akar-core`, immediately before
/// shaping — components never borrow a family name out of the font registry.
#[allow(dead_code)]
pub(crate) fn resolved_to_font_request(rt: &ResolvedTextStyle) -> akar_core::FontRequest {
    let family = match rt.font_family {
        FontFamily::SansSerif => akar_core::FontSelection::SansSerif,
        FontFamily::Serif => akar_core::FontSelection::Serif,
        FontFamily::Monospace => akar_core::FontSelection::Monospace,
        FontFamily::Named(handle) => akar_core::FontSelection::Named(handle),
    };
    let weight = match rt.font_weight {
        FontWeight::Normal => glyphon::Weight::NORMAL,
        FontWeight::Medium => glyphon::Weight::MEDIUM,
        FontWeight::Semibold => glyphon::Weight::SEMIBOLD,
        FontWeight::Bold => glyphon::Weight::BOLD,
    };
    akar_core::FontRequest {
        family,
        weight: weight.0,
    }
}

#[allow(dead_code)]
pub(crate) fn resolved_to_metrics(rt: &ResolvedTextStyle) -> glyphon::Metrics {
    glyphon::Metrics::new(rt.font_size, rt.line_height)
}

/// Maps a logical (direction-independent) `TextAlign` through
/// `akar_layout::AkarDirection` to the physical `glyphon::cosmic_text::Align` cosmic-text
/// needs to align a shaped run inside its own full-width buffer.
///
/// `Start`/`End` are logical: in LTR they mean `Left`/`Right`, and in RTL
/// they *swap* to `Right`/`Left`. `Center` is direction-independent.
///
/// This mapping exists so components pass an explicit physical alignment
/// into `TextPipeline::set_text_styled` and let cosmic-text own alignment
/// inside the full-width buffer — instead of computing a manual x-offset on
/// top of cosmic-text's own (already direction-aware) default alignment,
/// which double-applies the offset for RTL content. See Epic 023 Task 9.
pub(crate) fn resolve_align(
    align: TextAlign,
    direction: akar_layout::AkarDirection,
) -> glyphon::cosmic_text::Align {
    use akar_layout::AkarDirection;

    match (align, direction) {
        (TextAlign::Start, AkarDirection::Ltr) => glyphon::cosmic_text::Align::Left,
        (TextAlign::Start, AkarDirection::Rtl) => glyphon::cosmic_text::Align::Right,
        (TextAlign::End, AkarDirection::Ltr) => glyphon::cosmic_text::Align::Right,
        (TextAlign::End, AkarDirection::Rtl) => glyphon::cosmic_text::Align::Left,
        (TextAlign::Center, _) => glyphon::cosmic_text::Align::Center,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use akar_layout::AkarDirection;

    #[test]
    fn resolve_align_covers_all_six_align_direction_mappings() {
        assert_eq!(
            resolve_align(TextAlign::Start, AkarDirection::Ltr),
            glyphon::cosmic_text::Align::Left
        );
        assert_eq!(
            resolve_align(TextAlign::End, AkarDirection::Ltr),
            glyphon::cosmic_text::Align::Right
        );
        assert_eq!(
            resolve_align(TextAlign::Center, AkarDirection::Ltr),
            glyphon::cosmic_text::Align::Center
        );
        assert_eq!(
            resolve_align(TextAlign::Start, AkarDirection::Rtl),
            glyphon::cosmic_text::Align::Right,
            "Start is logical: it swaps to the physical right edge under RTL"
        );
        assert_eq!(
            resolve_align(TextAlign::End, AkarDirection::Rtl),
            glyphon::cosmic_text::Align::Left,
            "End is logical: it swaps to the physical left edge under RTL"
        );
        assert_eq!(
            resolve_align(TextAlign::Center, AkarDirection::Rtl),
            glyphon::cosmic_text::Align::Center,
            "Center is direction-independent"
        );
    }
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
    fn resolved_to_font_request_maps_family_and_weight() {
        let rt = ResolvedTextStyle {
            font_size: 16.0,
            line_height: 20.0,
            color: 0xff,
            font_weight: FontWeight::Bold,
            font_family: FontFamily::Serif,
            align: TextAlign::Start,
            wrap: false,
        };

        let request = resolved_to_font_request(&rt);

        assert_eq!(request.weight, glyphon::Weight::BOLD.0);
        assert_eq!(request.family, akar_core::FontSelection::Serif);
    }

    #[test]
    fn font_weight_enum_maps_to_glyphon_weights() {
        let sans = resolved_to_font_request(&ResolvedTextStyle {
            font_size: 16.0,
            line_height: 20.0,
            color: 0,
            font_weight: FontWeight::Normal,
            font_family: FontFamily::SansSerif,
            align: TextAlign::Start,
            wrap: false,
        });
        assert_eq!(sans.weight, glyphon::Weight::NORMAL.0);
        assert_eq!(sans.family, akar_core::FontSelection::SansSerif);

        let mono = resolved_to_font_request(&ResolvedTextStyle {
            font_size: 16.0,
            line_height: 20.0,
            color: 0,
            font_weight: FontWeight::Medium,
            font_family: FontFamily::Monospace,
            align: TextAlign::Start,
            wrap: false,
        });
        assert_eq!(mono.weight, glyphon::Weight::MEDIUM.0);
        assert_eq!(mono.family, akar_core::FontSelection::Monospace);

        let semi = resolved_to_font_request(&ResolvedTextStyle {
            font_size: 16.0,
            line_height: 20.0,
            color: 0,
            font_weight: FontWeight::Semibold,
            font_family: FontFamily::SansSerif,
            align: TextAlign::Start,
            wrap: false,
        });
        assert_eq!(semi.weight, glyphon::Weight::SEMIBOLD.0);
    }

    #[test]
    fn named_family_maps_to_handle_selection() {
        let request = resolved_to_font_request(&ResolvedTextStyle {
            font_size: 16.0,
            line_height: 20.0,
            color: 0,
            font_weight: FontWeight::Normal,
            font_family: FontFamily::Named(7),
            align: TextAlign::Start,
            wrap: false,
        });

        assert_eq!(request.family, akar_core::FontSelection::Named(7));
    }

    #[test]
    fn named_family_survives_style_cascade() {
        let theme = AKAR_THEME_DARK;
        let defaults = h1_defaults(&theme);
        let override_style = TextStyle {
            font_family: Some(FontFamily::Named(3)),
            ..TextStyle::empty()
        };

        let resolved = resolve_text_style(&theme, &defaults, Some(&override_style));

        assert_eq!(resolved.font_family, FontFamily::Named(3));
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
