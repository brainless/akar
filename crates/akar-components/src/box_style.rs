use crate::AkarTheme;

#[derive(Clone, Copy, Debug)]
pub struct BoxShadow {
    pub color: u32,
    pub offset: [f32; 2],
    pub blur: f32,
    pub spread: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct BoxStyle {
    pub fill: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub corner_radii: [f32; 4],
    pub shadow: Option<BoxShadow>,
}

impl BoxStyle {
    pub fn flat(fill: u32) -> Self {
        Self {
            fill,
            border_color: 0,
            border_width: 0.0,
            corner_radii: [0.0; 4],
            shadow: None,
        }
    }

    pub fn surface(theme: &AkarTheme) -> Self {
        Self {
            fill: theme.base_100,
            border_color: 0,
            border_width: 0.0,
            corner_radii: [theme.radius_box; 4],
            shadow: None,
        }
    }

    pub fn panel(theme: &AkarTheme) -> Self {
        Self {
            fill: theme.base_200,
            border_color: theme.base_300,
            border_width: theme.border_width,
            corner_radii: [theme.radius_box; 4],
            shadow: None,
        }
    }

    pub fn card(theme: &AkarTheme) -> Self {
        Self {
            fill: theme.base_100,
            border_color: theme.base_300,
            border_width: theme.border_width,
            corner_radii: [theme.radius_box; 4],
            shadow: Some(BoxShadow {
                color: 0x00000040,
                offset: [0.0, 4.0],
                blur: 12.0,
                spread: 0.0,
            }),
        }
    }
}
