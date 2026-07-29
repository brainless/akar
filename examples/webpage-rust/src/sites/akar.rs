use akar_components::{
    akar_badge, akar_badge_styled, akar_button, akar_card, akar_container, akar_heading, akar_link,
    akar_navbar, akar_paragraph, akar_separator, akar_stat, akar_tab_bar, scroll_area_begin,
    scroll_area_end, BadgeStyle, BadgeVariant, BoxStyle, ButtonVariant, CardSlots, CardStyle,
    FontFamily, HeadingLevel, NavbarStyle, TabVariant, TextStyle, AKAR_THEME_LIGHT,
};
use akar_core::AkarCore;
use akar_layout::{
    length, AlignItems, Dimension, Display, FlexDirection, JustifyContent, Layout, NodeId, Size,
    Style,
};

use crate::site::Site;

const THEME: akar_components::AkarTheme = AKAR_THEME_LIGHT;

pub struct AkarSite {
    root: NodeId,
    navbar_root: NodeId,
    logo_node: NodeId,
    feat_link: NodeId,
    comp_link: NodeId,
    github_link: NodeId,
    scroll_container: NodeId,
    scroll_y: f32,
    hero_root: NodeId,
    h1_node: NodeId,
    subtitle_node: NodeId,
    cta_solid: NodeId,
    cta_outline: NodeId,
    stats_root: NodeId,
    stat_nodes: [NodeId; 3],
    cards_root: NodeId,
    card_roots: [NodeId; 3],
    card_headings: [NodeId; 3],
    card_paras: [NodeId; 3],
    why_root: NodeId,
    why_h2: NodeId,
    why_body_para: NodeId,
    why_h4s: [NodeId; 4],
    why_paras: [NodeId; 4],
    showcase_root: NodeId,
    showcase_badges_h3: NodeId,
    badge_nodes: [NodeId; 7],
    showcase_buttons_h3: NodeId,
    btn_solid: NodeId,
    btn_outline: NodeId,
    btn_ghost: NodeId,
    showcase_tabs_h3: NodeId,
    tab_bar_node: NodeId,
    active_tab: usize,
    footer_root: NodeId,
    separator_node: NodeId,
    footer_col_headings: [NodeId; 3],
    footer_link_nodes: [NodeId; 6],
    copyright_label: NodeId,
}

impl AkarSite {
    pub fn new() -> Self {
        let n = || NodeId::new(0);
        Self {
            root: n(),
            navbar_root: n(),
            logo_node: n(),
            feat_link: n(),
            comp_link: n(),
            github_link: n(),
            scroll_container: n(),
            scroll_y: 0.0,
            hero_root: n(),
            h1_node: n(),
            subtitle_node: n(),
            cta_solid: n(),
            cta_outline: n(),
            stats_root: n(),
            stat_nodes: [n(); 3],
            cards_root: n(),
            card_roots: [n(); 3],
            card_headings: [n(); 3],
            card_paras: [n(); 3],
            why_root: n(),
            why_h2: n(),
            why_body_para: n(),
            why_h4s: [n(); 4],
            why_paras: [n(); 4],
            showcase_root: n(),
            showcase_badges_h3: n(),
            badge_nodes: [n(); 7],
            showcase_buttons_h3: n(),
            btn_solid: n(),
            btn_outline: n(),
            btn_ghost: n(),
            showcase_tabs_h3: n(),
            tab_bar_node: n(),
            active_tab: 0,
            footer_root: n(),
            separator_node: n(),
            footer_col_headings: [n(); 3],
            footer_link_nodes: [n(); 6],
            copyright_label: n(),
        }
    }
}

impl Site for AkarSite {
    fn name(&self) -> &str {
        "akar"
    }

    fn root(&self) -> NodeId {
        self.root
    }

    fn build_layout(&mut self, layout: &mut Layout) {
        let root = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        });

        let navbar_root = layout.new_leaf(Style::default());
        let logo_node = layout.new_leaf(Style {
            size: Size {
                width: length(120.0f32),
                height: length(40.0f32),
            },
            ..Default::default()
        });
        let feat_link = layout.new_leaf(Style {
            size: Size {
                width: length(100.0f32),
                height: length(24.0f32),
            },
            ..Default::default()
        });
        let comp_link = layout.new_leaf(Style {
            size: Size {
                width: length(120.0f32),
                height: length(24.0f32),
            },
            ..Default::default()
        });
        let github_link = layout.new_leaf(Style {
            size: Size {
                width: length(80.0f32),
                height: length(24.0f32),
            },
            ..Default::default()
        });

        layout.set_children(navbar_root, &[logo_node, feat_link, comp_link, github_link]);
        layout.set_style(
            navbar_root,
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: Some(AlignItems::CENTER),
                justify_content: Some(JustifyContent::SPACE_BETWEEN),
                flex_shrink: 0.0,
                size: Size {
                    width: Dimension::percent(1.0),
                    height: length(64.0f32),
                },
                padding: taffy::geometry::Rect {
                    left: length(48.0f32),
                    right: length(48.0f32),
                    top: length(0.0f32),
                    bottom: length(0.0f32),
                },
                ..Default::default()
            },
        );

        let scroll_container = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            ..Default::default()
        });

        let hero_root = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: Some(AlignItems::CENTER),
            justify_content: Some(JustifyContent::CENTER),
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(480.0f32),
            },
            gap: taffy::geometry::Size {
                width: length(0.0f32),
                height: length(16.0f32),
            },
            ..Default::default()
        });
        let h1_node = layout.new_leaf(Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: length(60.0f32),
            },
            ..Default::default()
        });
        let subtitle_node = layout.new_leaf(Style {
            size: Size {
                width: length(600.0f32),
                height: length(48.0f32),
            },
            ..Default::default()
        });
        let cta_solid = layout.new_leaf(Style {
            size: Size {
                width: length(160.0f32),
                height: length(44.0f32),
            },
            ..Default::default()
        });
        let cta_outline = layout.new_leaf(Style {
            size: Size {
                width: length(160.0f32),
                height: length(44.0f32),
            },
            ..Default::default()
        });
        layout.set_children(hero_root, &[h1_node, subtitle_node, cta_solid, cta_outline]);

        let stats_root = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(160.0f32),
            },
            gap: taffy::geometry::Size {
                width: length(24.0f32),
                height: length(0.0f32),
            },
            padding: taffy::geometry::Rect {
                left: length(48.0f32),
                right: length(48.0f32),
                top: length(0.0f32),
                bottom: length(0.0f32),
            },
            ..Default::default()
        });
        let stat_nodes: [NodeId; 3] = std::array::from_fn(|_| {
            layout.new_leaf(Style {
                flex_grow: 1.0,
                size: Size {
                    width: Dimension::auto(),
                    height: length(140.0f32),
                },
                ..Default::default()
            })
        });
        layout.set_children(stats_root, &stat_nodes);

        let cards_root = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(280.0f32),
            },
            gap: taffy::geometry::Size {
                width: length(24.0f32),
                height: length(0.0f32),
            },
            padding: taffy::geometry::Rect {
                left: length(48.0f32),
                right: length(48.0f32),
                top: length(0.0f32),
                bottom: length(0.0f32),
            },
            ..Default::default()
        });
        let mut card_roots = [NodeId::new(0); 3];
        let mut card_headings = [NodeId::new(0); 3];
        let mut card_paras = [NodeId::new(0); 3];
        for i in 0..3 {
            let card_root = layout.new_leaf(Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                size: Size {
                    width: Dimension::auto(),
                    height: length(260.0f32),
                },
                padding: taffy::geometry::Rect {
                    left: length(24.0f32),
                    right: length(24.0f32),
                    top: length(28.0f32),
                    bottom: length(24.0f32),
                },
                gap: taffy::geometry::Size {
                    width: length(0.0f32),
                    height: length(8.0f32),
                },
                ..Default::default()
            });
            let heading = layout.new_leaf(Style::default());
            let para = layout.new_leaf(Style {
                size: Size {
                    width: Dimension::percent(1.0),
                    height: length(120.0f32),
                },
                ..Default::default()
            });
            layout.set_children(card_root, &[heading, para]);
            card_roots[i] = card_root;
            card_headings[i] = heading;
            card_paras[i] = para;
        }
        layout.set_children(cards_root, &card_roots);

        let why_root = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            gap: taffy::geometry::Size {
                width: length(0.0f32),
                height: length(12.0f32),
            },
            padding: taffy::geometry::Rect {
                left: length(48.0f32),
                right: length(48.0f32),
                top: length(48.0f32),
                bottom: length(48.0f32),
            },
            ..Default::default()
        });
        let why_h2 = layout.new_leaf(Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: length(40.0f32),
            },
            ..Default::default()
        });
        let why_body_para = layout.new_leaf(Style {
            size: Size {
                width: length(700.0f32),
                height: length(48.0f32),
            },
            ..Default::default()
        });
        let mut why_h4s = [NodeId::new(0); 4];
        let mut why_paras = [NodeId::new(0); 4];
        for i in 0..4 {
            why_h4s[i] = layout.new_leaf(Style {
                size: Size {
                    width: Dimension::percent(1.0),
                    height: length(28.0f32),
                },
                ..Default::default()
            });
            why_paras[i] = layout.new_leaf(Style {
                size: Size {
                    width: length(700.0f32),
                    height: length(40.0f32),
                },
                ..Default::default()
            });
        }
        let mut why_children = Vec::with_capacity(10);
        why_children.push(why_h2);
        why_children.push(why_body_para);
        for i in 0..4 {
            why_children.push(why_h4s[i]);
            why_children.push(why_paras[i]);
        }
        layout.set_children(why_root, &why_children);

        let showcase_root = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            gap: taffy::geometry::Size {
                width: length(0.0f32),
                height: length(16.0f32),
            },
            padding: taffy::geometry::Rect {
                left: length(48.0f32),
                right: length(48.0f32),
                top: length(48.0f32),
                bottom: length(48.0f32),
            },
            ..Default::default()
        });
        let showcase_badges_h3 = layout.new_leaf(Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: length(32.0f32),
            },
            ..Default::default()
        });
        let badges_row = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_shrink: 0.0,
            gap: taffy::geometry::Size {
                width: length(8.0f32),
                height: length(0.0f32),
            },
            ..Default::default()
        });
        let badge_nodes: [NodeId; 7] = std::array::from_fn(|_| {
            layout.new_leaf(Style {
                size: Size {
                    width: length(80.0f32),
                    height: length(28.0f32),
                },
                ..Default::default()
            })
        });
        layout.set_children(badges_row, &badge_nodes);

        let showcase_buttons_h3 = layout.new_leaf(Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: length(32.0f32),
            },
            ..Default::default()
        });
        let buttons_row = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_shrink: 0.0,
            gap: taffy::geometry::Size {
                width: length(12.0f32),
                height: length(0.0f32),
            },
            ..Default::default()
        });
        let btn_solid = layout.new_leaf(Style {
            size: Size {
                width: length(120.0f32),
                height: length(40.0f32),
            },
            ..Default::default()
        });
        let btn_outline = layout.new_leaf(Style {
            size: Size {
                width: length(120.0f32),
                height: length(40.0f32),
            },
            ..Default::default()
        });
        let btn_ghost = layout.new_leaf(Style {
            size: Size {
                width: length(120.0f32),
                height: length(40.0f32),
            },
            ..Default::default()
        });
        layout.set_children(buttons_row, &[btn_solid, btn_outline, btn_ghost]);

        let showcase_tabs_h3 = layout.new_leaf(Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: length(32.0f32),
            },
            ..Default::default()
        });
        let tab_bar_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(40.0f32),
            },
            ..Default::default()
        });

        layout.set_children(
            showcase_root,
            &[
                showcase_badges_h3,
                badges_row,
                showcase_buttons_h3,
                buttons_row,
                showcase_tabs_h3,
                tab_bar_node,
            ],
        );

        let footer_root = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            gap: taffy::geometry::Size {
                width: length(0.0f32),
                height: length(24.0f32),
            },
            padding: taffy::geometry::Rect {
                left: length(48.0f32),
                right: length(48.0f32),
                top: length(48.0f32),
                bottom: length(48.0f32),
            },
            ..Default::default()
        });
        let separator_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(2.0f32),
            },
            ..Default::default()
        });

        let footer_columns = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_shrink: 0.0,
            gap: taffy::geometry::Size {
                width: length(48.0f32),
                height: length(0.0f32),
            },
            ..Default::default()
        });

        let col_gap = taffy::geometry::Size {
            width: length(0.0f32),
            height: length(8.0f32),
        };

        let col_product = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            gap: col_gap,
            ..Default::default()
        });
        let col_resources = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            gap: col_gap,
            ..Default::default()
        });
        let col_community = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            gap: col_gap,
            ..Default::default()
        });
        layout.set_children(footer_columns, &[col_product, col_resources, col_community]);

        let col_h_style = || Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: length(24.0f32),
            },
            ..Default::default()
        };
        let link_style = || Style {
            size: Size {
                width: length(160.0f32),
                height: length(24.0f32),
            },
            ..Default::default()
        };

        let mut footer_col_headings = [NodeId::new(0); 3];
        let mut footer_link_nodes = [NodeId::new(0); 6];

        footer_col_headings[0] = layout.new_leaf(col_h_style());
        footer_link_nodes[0] = layout.new_leaf(link_style());
        footer_link_nodes[1] = layout.new_leaf(link_style());
        footer_link_nodes[2] = layout.new_leaf(link_style());
        layout.set_children(
            col_product,
            &[
                footer_col_headings[0],
                footer_link_nodes[0],
                footer_link_nodes[1],
                footer_link_nodes[2],
            ],
        );

        footer_col_headings[1] = layout.new_leaf(col_h_style());
        footer_link_nodes[3] = layout.new_leaf(link_style());
        footer_link_nodes[4] = layout.new_leaf(link_style());
        layout.set_children(
            col_resources,
            &[
                footer_col_headings[1],
                footer_link_nodes[3],
                footer_link_nodes[4],
            ],
        );

        footer_col_headings[2] = layout.new_leaf(col_h_style());
        footer_link_nodes[5] = layout.new_leaf(link_style());
        layout.set_children(
            col_community,
            &[footer_col_headings[2], footer_link_nodes[5]],
        );

        let copyright_label = layout.new_leaf(Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: length(24.0f32),
            },
            ..Default::default()
        });
        layout.set_children(
            footer_root,
            &[separator_node, footer_columns, copyright_label],
        );

        layout.set_children(
            scroll_container,
            &[
                hero_root,
                stats_root,
                cards_root,
                why_root,
                showcase_root,
                footer_root,
            ],
        );

        layout.set_children(root, &[navbar_root, scroll_container]);

        self.root = root;
        self.navbar_root = navbar_root;
        self.logo_node = logo_node;
        self.feat_link = feat_link;
        self.comp_link = comp_link;
        self.github_link = github_link;
        self.scroll_container = scroll_container;
        self.hero_root = hero_root;
        self.h1_node = h1_node;
        self.subtitle_node = subtitle_node;
        self.cta_solid = cta_solid;
        self.cta_outline = cta_outline;
        self.stats_root = stats_root;
        self.stat_nodes = stat_nodes;
        self.cards_root = cards_root;
        self.card_roots = card_roots;
        self.card_headings = card_headings;
        self.card_paras = card_paras;
        self.why_root = why_root;
        self.why_h2 = why_h2;
        self.why_body_para = why_body_para;
        self.why_h4s = why_h4s;
        self.why_paras = why_paras;
        self.showcase_root = showcase_root;
        self.showcase_badges_h3 = showcase_badges_h3;
        self.badge_nodes = badge_nodes;
        self.showcase_buttons_h3 = showcase_buttons_h3;
        self.btn_solid = btn_solid;
        self.btn_outline = btn_outline;
        self.btn_ghost = btn_ghost;
        self.showcase_tabs_h3 = showcase_tabs_h3;
        self.tab_bar_node = tab_bar_node;
        self.footer_root = footer_root;
        self.separator_node = separator_node;
        self.footer_col_headings = footer_col_headings;
        self.footer_link_nodes = footer_link_nodes;
        self.copyright_label = copyright_label;
    }

    fn render(&mut self, core: &mut AkarCore, layout: &mut Layout, _viewport_rect: [f32; 4]) {
        akar_container(core, layout, self.root, &BoxStyle::flat(THEME.base_100));

        let navbar_style = NavbarStyle::default(&THEME);
        akar_navbar(core, layout, self.navbar_root, &navbar_style);
        akar_heading(
            core,
            layout,
            self.logo_node,
            "akar",
            HeadingLevel::H2,
            Some(TextStyle {
                font_size: Some(20.0),
                font_family: Some(FontFamily::Serif),
                ..TextStyle::empty()
            }),
            &THEME,
        );
        let _ = akar_link(core, layout, self.feat_link, "Features", None, &THEME);
        let _ = akar_link(core, layout, self.comp_link, "Components", None, &THEME);
        let _ = akar_link(core, layout, self.github_link, "GitHub", None, &THEME);

        let scroll_rect = layout.rect(self.scroll_container);
        let footer_rect = layout.rect(self.footer_root);
        let content_height = (footer_rect[1] + footer_rect[3]) - scroll_rect[1];
        let resp = scroll_area_begin(core, scroll_rect, &mut self.scroll_y, content_height);

        layout.set_screen_origin([0.0, resp.content_y - scroll_rect[1]]);

        akar_heading(
            core,
            layout,
            self.h1_node,
            "akar",
            HeadingLevel::H1,
            Some(TextStyle {
                font_size: Some(48.0),
                font_family: Some(FontFamily::Serif),
                ..TextStyle::empty()
            }),
            &THEME,
        );
        akar_paragraph(
            core,
            layout,
            self.subtitle_node,
            "A GPU-accelerated, language-neutral UI component library built on wgpu and glyphon.",
            None,
            &THEME,
        );
        let _ = akar_button(
            core,
            layout,
            self.cta_solid,
            "Get Started",
            ButtonVariant::Solid,
            &THEME,
        );
        let _ = akar_button(
            core,
            layout,
            self.cta_outline,
            "View on GitHub",
            ButtonVariant::Outline,
            &THEME,
        );

        akar_stat(
            core,
            layout,
            self.stat_nodes[0],
            "Components",
            "30+",
            None,
            &THEME,
        );
        akar_stat(
            core,
            layout,
            self.stat_nodes[1],
            "Language Neutral",
            "C ABI",
            None,
            &THEME,
        );
        akar_stat(
            core,
            layout,
            self.stat_nodes[2],
            "No Framework Opinions",
            "Immediate Mode",
            None,
            &THEME,
        );

        let card_style = CardStyle::default(&THEME);
        for i in 0..3 {
            let slots = CardSlots {
                header: None,
                body: self.card_roots[i],
                footer: None,
            };
            akar_card(core, layout, self.card_roots[i], &slots, &card_style);
        }
        let card_titles = [
            "Cross-Platform GPU Rendering",
            "Language Neutral C ABI",
            "Composable Components",
        ];
        let card_descriptions = [
            "Built on wgpu for native performance across macOS, Windows, and Linux. Every pixel is GPU-accelerated.",
            "Use from any language that calls C. The generated header is the only contract you need.",
            "Buttons, cards, inputs, and tables styled out of the box with semantic theme tokens.",
        ];
        for i in 0..3 {
            akar_heading(
                core,
                layout,
                self.card_headings[i],
                card_titles[i],
                HeadingLevel::H3,
                None,
                &THEME,
            );
            akar_paragraph(
                core,
                layout,
                self.card_paras[i],
                card_descriptions[i],
                None,
                &THEME,
            );
        }

        akar_heading(
            core,
            layout,
            self.why_h2,
            "Why akar",
            HeadingLevel::H2,
            None,
            &THEME,
        );
        akar_paragraph(
            core,
            layout,
            self.why_body_para,
            "akar is designed for agents and developers who need a fast, composable UI framework without framework opinions.",
            None,
            &THEME,
        );
        let why_titles = [
            "01. Built by agents, debuggable by agents",
            "02. Batteries-included component catalog",
            "03. Virtualization first",
            "04. Canvas LOD with component portals",
        ];
        let why_descriptions = [
            "akar ships with a full visual debug toolchain so agents can iterate without human intervention.",
            "A rich set of UI components with semantic theming and composable APIs.",
            "Scroll containers virtualize off-screen items for performance at any scale.",
            "LOD canvas rendering with full component interactivity through portal mode.",
        ];
        for i in 0..4 {
            akar_heading(
                core,
                layout,
                self.why_h4s[i],
                why_titles[i],
                HeadingLevel::H4,
                None,
                &THEME,
            );
            akar_paragraph(
                core,
                layout,
                self.why_paras[i],
                why_descriptions[i],
                None,
                &THEME,
            );
        }

        akar_heading(
            core,
            layout,
            self.showcase_badges_h3,
            "Badge Variants",
            HeadingLevel::H3,
            None,
            &THEME,
        );
        let badge_variants = [
            BadgeVariant::Default,
            BadgeVariant::Primary,
            BadgeVariant::Success,
            BadgeVariant::Warning,
            BadgeVariant::Error,
            BadgeVariant::Info,
            BadgeVariant::Primary,
        ];
        let badge_labels = [
            "Default", "Primary", "Success", "Warning", "Error", "Info", "Custom",
        ];
        for i in 0..6 {
            akar_badge(
                core,
                layout,
                self.badge_nodes[i],
                badge_labels[i],
                badge_variants[i],
                &THEME,
            );
        }
        let custom_badge_style = BadgeStyle {
            fill: Some(0x6366f1ff),
            content_color: Some(0xffffffff),
            ..BadgeStyle::empty()
        };
        akar_badge_styled(
            core,
            layout,
            self.badge_nodes[6],
            "Custom",
            BadgeVariant::Primary,
            &custom_badge_style,
            &THEME,
        );

        akar_heading(
            core,
            layout,
            self.showcase_buttons_h3,
            "Button Variants",
            HeadingLevel::H3,
            None,
            &THEME,
        );
        let _ = akar_button(
            core,
            layout,
            self.btn_solid,
            "Solid",
            ButtonVariant::Solid,
            &THEME,
        );
        let _ = akar_button(
            core,
            layout,
            self.btn_outline,
            "Outline",
            ButtonVariant::Outline,
            &THEME,
        );
        let _ = akar_button(
            core,
            layout,
            self.btn_ghost,
            "Ghost",
            ButtonVariant::Ghost,
            &THEME,
        );

        akar_heading(
            core,
            layout,
            self.showcase_tabs_h3,
            "Interactive Tab Bar",
            HeadingLevel::H3,
            None,
            &THEME,
        );
        let tab_resp = akar_tab_bar(
            core,
            layout,
            self.tab_bar_node,
            &["Overview", "Installation", "API Reference"],
            self.active_tab,
            TabVariant::Underline,
            &THEME,
        );
        if let Some(idx) = tab_resp.clicked {
            self.active_tab = idx;
        }

        akar_separator(core, layout, self.separator_node, &THEME);

        let col_headings = ["Product", "Resources", "Community"];
        for i in 0..3 {
            akar_heading(
                core,
                layout,
                self.footer_col_headings[i],
                col_headings[i],
                HeadingLevel::H4,
                None,
                &THEME,
            );
        }
        let link_texts = [
            "Features",
            "Components",
            "Demo",
            "GitHub",
            "Documentation",
            "MIT License",
        ];
        for i in 0..6 {
            let _ = akar_link(
                core,
                layout,
                self.footer_link_nodes[i],
                link_texts[i],
                None,
                &THEME,
            );
        }

        akar_paragraph(
            core,
            layout,
            self.copyright_label,
            "akar - MIT License",
            Some(TextStyle {
                color: Some(THEME.muted_content),
                ..TextStyle::empty()
            }),
            &THEME,
        );

        scroll_area_end(core);
        layout.set_screen_origin([0.0, 0.0]);
    }
}
