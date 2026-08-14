use thiserror::Error;

#[derive(Debug, Error)]
pub enum FontLoadError {
    #[error("font data could not be parsed: {0}")]
    InvalidFontData(String),
    #[error("no faces found in font data")]
    EmptyFontSource,
    /// v1 restricts loading to sources containing exactly one distinct family.
    /// A .ttc/.otc spanning more than one family hits this; a single .ttf/.otf
    /// never does.
    #[error("font data contains {0} distinct families; v1 requires exactly one")]
    MultipleFamilies(usize),
}

/// How `TextPipeline` populates its `fontdb::Database` at construction time.
///
/// Default is `Bundled`: system font scanning is opt-in because it is slow and
/// resolves to machine-dependent faces, which breaks akar's reproducible
/// screenshot guarantee.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontSource {
    /// Load only the fonts carried in `TextPipelineConfig::fonts`. Deterministic
    /// across machines.
    ///
    /// At least one font must be present: cosmic-text panics with
    /// "no default font found" on the first shaping call against an empty
    /// database. Missing script *coverage* within a present font degrades
    /// gracefully to tofu.
    #[default]
    Bundled,
    /// `TextPipelineConfig::fonts` plus a full system font scan. Not
    /// reproducible across machines or operating systems. Opt-in only.
    BundledPlusSystemScan,
}

#[cfg(feature = "bundled-font")]
pub const IBM_PLEX_SANS_REGULAR: &[u8] = include_bytes!("../assets/fonts/IBMPlexSans-Regular.ttf");

#[cfg(feature = "bundled-font")]
pub const IBM_PLEX_SANS_SEMIBOLD: &[u8] =
    include_bytes!("../assets/fonts/IBMPlexSans-SemiBold.ttf");

/// Font database configuration for `TextPipeline::new`.
///
/// `Default` seeds `fonts` with the bundled IBM Plex Sans faces when the
/// default-on `bundled-font` feature is enabled. With that feature disabled,
/// `fonts` is empty and the caller **must** supply at least one font before any
/// text is shaped; `TextPipeline::new` panics with an akar-side message
/// otherwise.
#[derive(Clone, Debug)]
pub struct TextPipelineConfig {
    pub font_source: FontSource,
    /// Font file bytes loaded before any text is shaped. Each entry may contain
    /// multiple faces (TTC/OTC); all are loaded.
    pub fonts: Vec<Vec<u8>>,
}

impl Default for TextPipelineConfig {
    fn default() -> Self {
        Self {
            font_source: FontSource::default(),
            fonts: bundled_fonts(),
        }
    }
}

impl TextPipelineConfig {
    /// Bundled default fonts, no system scan.
    pub fn bundled() -> Self {
        Self::default()
    }

    /// Bundled default fonts plus a system font scan. Not reproducible across
    /// machines — see `FontSource::BundledPlusSystemScan`.
    pub fn bundled_plus_system_scan() -> Self {
        Self {
            font_source: FontSource::BundledPlusSystemScan,
            fonts: bundled_fonts(),
        }
    }
}

#[cfg(feature = "bundled-font")]
pub fn bundled_fonts() -> Vec<Vec<u8>> {
    vec![
        IBM_PLEX_SANS_REGULAR.to_vec(),
        IBM_PLEX_SANS_SEMIBOLD.to_vec(),
    ]
}

#[cfg(not(feature = "bundled-font"))]
pub fn bundled_fonts() -> Vec<Vec<u8>> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_source_is_bundled() {
        assert_eq!(
            TextPipelineConfig::default().font_source,
            FontSource::Bundled
        );
    }

    #[test]
    fn system_scan_config_keeps_bundled_fonts() {
        let config = TextPipelineConfig::bundled_plus_system_scan();
        assert_eq!(config.font_source, FontSource::BundledPlusSystemScan);
        assert_eq!(config.fonts.len(), bundled_fonts().len());
    }

    #[cfg(feature = "bundled-font")]
    #[test]
    fn bundled_feature_supplies_two_faces() {
        assert_eq!(bundled_fonts().len(), 2);
        assert!(bundled_fonts().iter().all(|bytes| !bytes.is_empty()));
    }

    #[cfg(not(feature = "bundled-font"))]
    #[test]
    fn without_bundled_feature_default_config_is_empty() {
        assert!(TextPipelineConfig::default().fonts.is_empty());
    }
}
