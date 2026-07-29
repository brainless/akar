use akar_components::{scroll_area_begin, scroll_area_end};
use akar_core::AkarCore;
use akar_layout::{
    length, Dimension, Display, FlexDirection, JustifyContent, Layout, NodeId, Size, Style,
};

use crate::site::Site;

const THEME_BG: u32 = 0xfafafaff;
const THEME_TEXT: u32 = 0x09090bff;
const THEME_TEXT_SECONDARY: u32 = 0x52525bff;
const THEME_HERO_BG: u32 = 0xf4f4f5ff;
const THEME_CARD_BG: u32 = 0xffffffff;
const THEME_PATTERN: u32 = 0xd4d4d8ff;

fn hex_to_f4(c: u32) -> [f32; 4] {
    [
        ((c >> 24) & 0xFF) as f32 / 255.0,
        ((c >> 16) & 0xFF) as f32 / 255.0,
        ((c >> 8) & 0xFF) as f32 / 255.0,
        (c & 0xFF) as f32 / 255.0,
    ]
}

pub struct MimoSite {
    root: NodeId,
    header_node: NodeId,
    hero_node: NodeId,
    scroll_container: NodeId,
    scroll_y: f32,
    card_nodes: [NodeId; 3],
    build_section: NodeId,
    paper_section: NodeId,
}

impl MimoSite {
    pub fn new() -> Self {
        Self {
            root: NodeId::new(0),
            header_node: NodeId::new(0),
            hero_node: NodeId::new(0),
            scroll_container: NodeId::new(0),
            scroll_y: 0.0,
            card_nodes: [NodeId::new(0); 3],
            build_section: NodeId::new(0),
            paper_section: NodeId::new(0),
        }
    }

    fn render_header(core: &mut AkarCore, header_rect: [f32; 4]) {
        let bg = hex_to_f4(THEME_BG);
        core.draw_list.push_quad(akar_core::QuadCall {
            rect: header_rect,
            fill: bg,
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let logo_buf = core.text_pipeline.set_text(
            Some(100),
            "Xiaomi MiMo",
            glyphon::Metrics::new(20.0, 20.0 * 1.3),
            None,
            None,
            None,
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: logo_buf,
            x: header_rect[0],
            y: header_rect[1] + 14.0,
            clip: header_rect,
            color: hex_to_f4(THEME_TEXT),
            z: 0.0,
        });

        let nav_items = ["Product", "Research", "Join Us"];
        let mut x_offset = header_rect[0] + header_rect[2] - 20.0;
        for (i, item) in nav_items.iter().rev().enumerate() {
            let buf = core.text_pipeline.set_text(
                Some(200 + i as u64),
                item,
                glyphon::Metrics::new(15.0, 15.0 * 1.4),
                None,
                None,
                None,
            );
            let measured = core.text_pipeline.measure(buf, None);
            let text_w = measured.x;
            x_offset -= text_w;
            core.draw_list.push_text(akar_core::TextCall {
                buffer_id: buf,
                x: x_offset,
                y: header_rect[1] + 18.0,
                clip: header_rect,
                color: hex_to_f4(THEME_TEXT),
                z: 0.0,
            });
            x_offset -= 32.0;
        }
    }

    fn render_hero(core: &mut AkarCore, hero_rect: [f32; 4]) {
        let bg = hex_to_f4(THEME_HERO_BG);
        core.draw_list.push_quad(akar_core::QuadCall {
            rect: hero_rect,
            fill: bg,
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let pattern_color = hex_to_f4(THEME_PATTERN);
        let pattern_text = "M I M O M I M O M I M O M I M O M I M O";
        let row_height = 52.0;
        let start_y = hero_rect[1] + 20.0;
        let mut row = 0u64;
        let mut y = start_y;
        while y < hero_rect[1] + hero_rect[3] - 20.0 {
            let x_offset = if row % 2 == 0 { 0.0 } else { -40.0 };
            let buf = core.text_pipeline.set_text(
                Some(3000 + row),
                pattern_text,
                glyphon::Metrics::new(28.0, 28.0 * 1.2),
                Some(hero_rect[2] + 80.0),
                None,
                None,
            );
            core.draw_list.push_text(akar_core::TextCall {
                buffer_id: buf,
                x: hero_rect[0] + x_offset,
                y,
                clip: hero_rect,
                color: pattern_color,
                z: 0.0,
            });
            y += row_height;
            row += 1;
        }

        let title_buf = core.text_pipeline.set_text(
            Some(4000),
            "HELLO, I'M MiMo",
            glyphon::Metrics::new(72.0, 72.0 * 1.1),
            None,
            None,
            Some(glyphon::Attrs::new().family(glyphon::Family::Serif)),
        );
        let measured = core.text_pipeline.measure(title_buf, None);
        let title_w = measured.x;
        let title_h = 80.0;
        let title_x = hero_rect[0] + (hero_rect[2] - title_w) / 2.0;
        let title_y = hero_rect[1] + (hero_rect[3] - title_h) / 2.0;
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: title_buf,
            x: title_x,
            y: title_y,
            clip: hero_rect,
            color: hex_to_f4(THEME_TEXT),
            z: 0.1,
        });
    }

    fn render_card(core: &mut AkarCore, card_rect: [f32; 4], index: usize) {
        let bg = hex_to_f4(THEME_CARD_BG);
        core.draw_list.push_quad(akar_core::QuadCall {
            rect: card_rect,
            fill: bg,
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let (title, subtitle) = match index {
            0 => (
                "Xiaomi MiMo-V2.5-Pro",
                "A leap in agentic and long horizon coherence.",
            ),
            1 => ("Xiaomi MiMo-V2.5", "A leap in agency and multimodality."),
            2 => (
                "Xiaomi MiMo-V2.5-TTS Series",
                "Give your agent a voice. Give it a soul.",
            ),
            _ => ("", ""),
        };

        let title_buf = core.text_pipeline.set_text(
            Some(5000 + index as u64),
            title,
            glyphon::Metrics::new(20.0, 20.0 * 1.3),
            Some(card_rect[2] - 40.0),
            None,
            None,
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: title_buf,
            x: card_rect[0] + 20.0,
            y: card_rect[1] + 24.0,
            clip: card_rect,
            color: hex_to_f4(THEME_TEXT),
            z: 0.0,
        });

        let sub_buf = core.text_pipeline.set_text(
            Some(5100 + index as u64),
            subtitle,
            glyphon::Metrics::new(14.0, 14.0 * 1.4),
            Some(card_rect[2] - 40.0),
            None,
            None,
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: sub_buf,
            x: card_rect[0] + 20.0,
            y: card_rect[1] + 56.0,
            clip: card_rect,
            color: hex_to_f4(THEME_TEXT_SECONDARY),
            z: 0.0,
        });

        let wave_color = hex_to_f4(THEME_PATTERN);
        let wave_y_base = card_rect[1] + card_rect[3] - 60.0;
        for w in 0..3 {
            let wave_rect = [
                card_rect[0] + 20.0 + w as f32 * 50.0,
                wave_y_base + (w as f32 * 8.0).sin() * 10.0,
                60.0,
                40.0,
            ];
            core.draw_list.push_quad(akar_core::QuadCall {
                rect: wave_rect,
                fill: [wave_color[0], wave_color[1], wave_color[2], 0.3],
                border_color: [0.0; 4],
                corner_radii: [20.0; 4],
                border_width: 0.0,
                z: 0.0,
                shadow_blur: 0.0,
                shadow_spread: 0.0,
                shadow_color: [0.0; 4],
                shadow_offset: [0.0; 2],
                _pad: [0.0; 2],
            });
        }

        if index == 1 {
            let dot_rect = [
                card_rect[0] + card_rect[2] / 2.0 - 15.0,
                card_rect[1] + card_rect[3] - 50.0,
                30.0,
                30.0,
            ];
            core.draw_list.push_quad(akar_core::QuadCall {
                rect: dot_rect,
                fill: [wave_color[0], wave_color[1], wave_color[2], 0.4],
                border_color: [0.0; 4],
                corner_radii: [15.0; 4],
                border_width: 0.0,
                z: 0.0,
                shadow_blur: 0.0,
                shadow_spread: 0.0,
                shadow_color: [0.0; 4],
                shadow_offset: [0.0; 2],
                _pad: [0.0; 2],
            });
        }
    }

    fn render_build_section(core: &mut AkarCore, section_rect: [f32; 4]) {
        core.draw_list.push_quad(akar_core::QuadCall {
            rect: section_rect,
            fill: hex_to_f4(THEME_BG),
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let heading_buf = core.text_pipeline.set_text(
            Some(6000),
            "Build with MiMo",
            glyphon::Metrics::new(42.0, 42.0 * 1.2),
            None,
            None,
            Some(glyphon::Attrs::new().family(glyphon::Family::Serif)),
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: heading_buf,
            x: section_rect[0] + 48.0,
            y: section_rect[1] + 48.0,
            clip: section_rect,
            color: hex_to_f4(THEME_TEXT),
            z: 0.0,
        });

        let desc_buf = core.text_pipeline.set_text(
            Some(6001),
            "Experience the powerful capabilities of Xiaomi MiMo's large-scale model\nnow and explore the infinite possibilities of AI.",
            glyphon::Metrics::new(16.0, 16.0 * 1.5),
            Some(400.0),
            None,
            None,
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: desc_buf,
            x: section_rect[0] + section_rect[2] * 0.45,
            y: section_rect[1] + 56.0,
            clip: section_rect,
            color: hex_to_f4(THEME_TEXT_SECONDARY),
            z: 0.0,
        });

        let items = [
            (
                "01",
                "Web Demo",
                "Interact with MiMo directly through the web",
            ),
            (
                "02",
                "API Access",
                "Developer portal for quick integration of MiMo capabilities",
            ),
        ];

        let row_height = 80.0;
        let content_top = section_rect[1] + 120.0;

        for (i, (num, title, desc)) in items.iter().enumerate() {
            let row_y = content_top + i as f32 * row_height;

            core.draw_list.push_quad(akar_core::QuadCall {
                rect: [section_rect[0], row_y, section_rect[2], row_height],
                fill: hex_to_f4(THEME_BG),
                border_color: [0.0; 4],
                corner_radii: [0.0; 4],
                border_width: 0.0,
                z: 0.0,
                shadow_blur: 0.0,
                shadow_spread: 0.0,
                shadow_color: [0.0; 4],
                shadow_offset: [0.0; 2],
                _pad: [0.0; 2],
            });

            let num_buf = core.text_pipeline.set_text(
                Some(6100 + i as u64),
                num,
                glyphon::Metrics::new(18.0, 18.0 * 1.3),
                Some(60.0),
                None,
                None,
            );
            core.draw_list.push_text(akar_core::TextCall {
                buffer_id: num_buf,
                x: section_rect[0] + 48.0,
                y: row_y + 20.0,
                clip: section_rect,
                color: hex_to_f4(THEME_TEXT_SECONDARY),
                z: 0.0,
            });

            let title_buf = core.text_pipeline.set_text(
                Some(6200 + i as u64),
                title,
                glyphon::Metrics::new(20.0, 20.0 * 1.3),
                Some(section_rect[2] * 0.5),
                None,
                None,
            );
            core.draw_list.push_text(akar_core::TextCall {
                buffer_id: title_buf,
                x: section_rect[0] + 120.0,
                y: row_y + 16.0,
                clip: section_rect,
                color: hex_to_f4(THEME_TEXT),
                z: 0.0,
            });

            let desc_buf = core.text_pipeline.set_text(
                Some(6300 + i as u64),
                desc,
                glyphon::Metrics::new(14.0, 14.0 * 1.4),
                Some(section_rect[2] * 0.5),
                None,
                None,
            );
            core.draw_list.push_text(akar_core::TextCall {
                buffer_id: desc_buf,
                x: section_rect[0] + 120.0,
                y: row_y + 44.0,
                clip: section_rect,
                color: hex_to_f4(THEME_TEXT_SECONDARY),
                z: 0.0,
            });

            let arrow_buf = core.text_pipeline.set_text(
                Some(6400 + i as u64),
                "\u{2192}",
                glyphon::Metrics::new(24.0, 24.0 * 1.3),
                None,
                None,
                None,
            );
            core.draw_list.push_text(akar_core::TextCall {
                buffer_id: arrow_buf,
                x: section_rect[0] + section_rect[2] - 60.0,
                y: row_y + 24.0,
                clip: section_rect,
                color: hex_to_f4(THEME_TEXT),
                z: 0.0,
            });
        }
    }

    fn render_paper_section(core: &mut AkarCore, section_rect: [f32; 4]) {
        core.draw_list.push_quad(akar_core::QuadCall {
            rect: section_rect,
            fill: hex_to_f4(THEME_BG),
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let heading_buf = core.text_pipeline.set_text(
            Some(7000),
            "Paper",
            glyphon::Metrics::new(42.0, 42.0 * 1.2),
            None,
            None,
            Some(glyphon::Attrs::new().family(glyphon::Family::Serif)),
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: heading_buf,
            x: section_rect[0] + 48.0,
            y: section_rect[1] + 48.0,
            clip: section_rect,
            color: hex_to_f4(THEME_TEXT),
            z: 0.0,
        });

        let items = [(
            "01",
            "MOPD: Multi-Teacher On-Policy Distillation for Capability Integration in LLM Post-Training",
            "June 29, 2026",
        )];

        let row_height = 100.0;
        let content_top = section_rect[1] + 110.0;

        for (i, (num, title, date)) in items.iter().enumerate() {
            let row_y = content_top + i as f32 * row_height;

            core.draw_list.push_quad(akar_core::QuadCall {
                rect: [section_rect[0], row_y, section_rect[2], row_height],
                fill: hex_to_f4(THEME_BG),
                border_color: [0.0; 4],
                corner_radii: [0.0; 4],
                border_width: 0.0,
                z: 0.0,
                shadow_blur: 0.0,
                shadow_spread: 0.0,
                shadow_color: [0.0; 4],
                shadow_offset: [0.0; 2],
                _pad: [0.0; 2],
            });

            let num_buf = core.text_pipeline.set_text(
                Some(7100 + i as u64),
                num,
                glyphon::Metrics::new(18.0, 18.0 * 1.3),
                Some(60.0),
                None,
                None,
            );
            core.draw_list.push_text(akar_core::TextCall {
                buffer_id: num_buf,
                x: section_rect[0] + 48.0,
                y: row_y + 28.0,
                clip: section_rect,
                color: hex_to_f4(THEME_TEXT_SECONDARY),
                z: 0.0,
            });

            let title_buf = core.text_pipeline.set_text(
                Some(7200 + i as u64),
                title,
                glyphon::Metrics::new(20.0, 20.0 * 1.3),
                Some(section_rect[2] * 0.6),
                None,
                None,
            );
            core.draw_list.push_text(akar_core::TextCall {
                buffer_id: title_buf,
                x: section_rect[0] + 120.0,
                y: row_y + 18.0,
                clip: section_rect,
                color: hex_to_f4(THEME_TEXT),
                z: 0.0,
            });

            let date_buf = core.text_pipeline.set_text(
                Some(7300 + i as u64),
                date,
                glyphon::Metrics::new(13.0, 13.0 * 1.4),
                Some(section_rect[2] * 0.6),
                None,
                None,
            );
            core.draw_list.push_text(akar_core::TextCall {
                buffer_id: date_buf,
                x: section_rect[0] + 120.0,
                y: row_y + 48.0,
                clip: section_rect,
                color: hex_to_f4(THEME_TEXT_SECONDARY),
                z: 0.0,
            });

            let arrow_buf = core.text_pipeline.set_text(
                Some(7400 + i as u64),
                "\u{2192}",
                glyphon::Metrics::new(24.0, 24.0 * 1.3),
                None,
                None,
                None,
            );
            core.draw_list.push_text(akar_core::TextCall {
                buffer_id: arrow_buf,
                x: section_rect[0] + section_rect[2] - 60.0,
                y: row_y + 32.0,
                clip: section_rect,
                color: hex_to_f4(THEME_TEXT),
                z: 0.0,
            });
        }
    }
}

impl Site for MimoSite {
    fn name(&self) -> &str {
        "mimo"
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
            padding: taffy::geometry::Rect {
                left: length(48.0f32),
                right: length(48.0f32),
                top: length(0.0f32),
                bottom: length(0.0f32),
            },
            ..Default::default()
        });

        let header_node = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: Some(akar_layout::AlignItems::CENTER),
            justify_content: Some(JustifyContent::SPACE_BETWEEN),
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(64.0f32),
            },
            padding: taffy::geometry::Rect {
                left: length(0.0f32),
                right: length(0.0f32),
                top: length(12.0f32),
                bottom: length(12.0f32),
            },
            ..Default::default()
        });

        let hero_node = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: Some(akar_layout::AlignItems::CENTER),
            justify_content: Some(JustifyContent::CENTER),
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(480.0f32),
            },
            ..Default::default()
        });

        let card_style = || Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            size: Size {
                width: Dimension::auto(),
                height: length(220.0f32),
            },
            padding: taffy::geometry::Rect {
                left: length(20.0f32),
                right: length(20.0f32),
                top: length(20.0f32),
                bottom: length(20.0f32),
            },
            ..Default::default()
        };

        let card_nodes = [
            layout.new_leaf(card_style()),
            layout.new_leaf(card_style()),
            layout.new_leaf(card_style()),
        ];

        let cards_row = layout.new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_shrink: 0.0,
                size: Size {
                    width: Dimension::percent(1.0),
                    height: length(220.0f32),
                },
                gap: taffy::geometry::Size {
                    width: length(24.0f32),
                    height: length(0.0f32),
                },
                margin: taffy::geometry::Rect {
                    top: length(0.0f32),
                    right: length(0.0f32),
                    bottom: length(0.0f32),
                    left: length(0.0f32),
                },
                ..Default::default()
            },
            &card_nodes,
        );

        let build_section = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(304.0f32),
            },
            margin: taffy::geometry::Rect {
                top: length(0.0f32),
                right: length(0.0f32),
                bottom: length(0.0f32),
                left: length(0.0f32),
            },
            ..Default::default()
        });

        let paper_section = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(210.0f32),
            },
            ..Default::default()
        });

        let scroll_container = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            ..Default::default()
        });

        layout.set_children(root, &[header_node, scroll_container]);
        layout.set_children(
            scroll_container,
            &[hero_node, cards_row, build_section, paper_section],
        );

        self.root = root;
        self.header_node = header_node;
        self.hero_node = hero_node;
        self.scroll_container = scroll_container;
        self.card_nodes = card_nodes;
        self.build_section = build_section;
        self.paper_section = paper_section;
    }

    fn render(&mut self, core: &mut AkarCore, layout: &Layout, viewport_rect: [f32; 4]) {
        let bg = hex_to_f4(THEME_BG);
        core.draw_list.push_quad(akar_core::QuadCall {
            rect: [0.0, 0.0, viewport_rect[2], viewport_rect[3]],
            fill: bg,
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let header_rect = layout.rect(self.header_node);
        Self::render_header(core, header_rect);

        let scroll_rect = layout.rect(self.scroll_container);
        let paper_rect = layout.rect(self.paper_section);
        let content_height = (paper_rect[1] + paper_rect[3]) - scroll_rect[1];
        let resp = scroll_area_begin(core, scroll_rect, &mut self.scroll_y, content_height);
        let offset_y = resp.content_y - scroll_rect[1];

        core.draw_list.push_quad(akar_core::QuadCall {
            rect: [
                scroll_rect[0],
                scroll_rect[1],
                scroll_rect[2],
                content_height,
            ],
            fill: bg,
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let hero_abs = layout.rect(self.hero_node);
        let build_abs = layout.rect(self.build_section);
        let gap_bg_top = hero_abs[1] + hero_abs[3];
        let gap_bg_height = build_abs[1] - gap_bg_top;
        if gap_bg_height > 0.0 {
            core.draw_list.push_quad(akar_core::QuadCall {
                rect: [
                    scroll_rect[0],
                    gap_bg_top + offset_y,
                    scroll_rect[2],
                    gap_bg_height,
                ],
                fill: bg,
                border_color: [0.0; 4],
                corner_radii: [0.0; 4],
                border_width: 0.0,
                z: 0.0,
                shadow_blur: 0.0,
                shadow_spread: 0.0,
                shadow_color: [0.0; 4],
                shadow_offset: [0.0; 2],
                _pad: [0.0; 2],
            });
        }

        let hero_rect = layout.rect(self.hero_node);
        let hero_rect = [
            hero_rect[0],
            hero_rect[1] + offset_y,
            hero_rect[2],
            hero_rect[3],
        ];
        Self::render_hero(core, hero_rect);

        let card_rects: Vec<[f32; 4]> = self.card_nodes.iter().map(|&n| layout.rect(n)).collect();

        let cards_with_y: Vec<[f32; 4]> = card_rects
            .iter()
            .map(|&r| [r[0], r[1] + offset_y, r[2], r[3]])
            .collect();
        let cards_left = cards_with_y
            .iter()
            .map(|r| r[0])
            .fold(f32::INFINITY, f32::min);
        let cards_top = cards_with_y
            .iter()
            .map(|r| r[1])
            .fold(f32::INFINITY, f32::min);
        let cards_right = cards_with_y
            .iter()
            .map(|r| r[0] + r[2])
            .fold(f32::NEG_INFINITY, f32::max);
        let cards_bottom = cards_with_y
            .iter()
            .map(|r| r[1] + r[3])
            .fold(f32::NEG_INFINITY, f32::max);
        core.draw_list.push_quad(akar_core::QuadCall {
            rect: [
                cards_left,
                cards_top,
                cards_right - cards_left,
                cards_bottom - cards_top,
            ],
            fill: bg,
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });
        for (i, card_rect) in card_rects.into_iter().enumerate() {
            let card_rect = [
                card_rect[0],
                card_rect[1] + offset_y,
                card_rect[2],
                card_rect[3],
            ];
            Self::render_card(core, card_rect, i);
        }

        let build_rect = layout.rect(self.build_section);
        let build_rect = [
            build_rect[0],
            build_rect[1] + offset_y,
            build_rect[2],
            build_rect[3],
        ];
        Self::render_build_section(core, build_rect);

        let paper_rect = [
            paper_rect[0],
            paper_rect[1] + offset_y,
            paper_rect[2],
            paper_rect[3],
        ];
        Self::render_paper_section(core, paper_rect);

        scroll_area_end(core);
    }
}
