use std::collections::HashMap;

use glam::DVec3;

use crate::entity::MetaValue;
use crate::entity::components::Position;
use crate::renderer::pipelines::menu_overlay::MenuElement;
use crate::ui::text::TextSpan;

#[allow(dead_code)]
pub struct TextDisplayEntity {
    pub id: i32,
    pub uuid: uuid::Uuid,
    pub position: Position,
    pub prev_position: Position,
    pub text: Vec<TextSpan>,
    pub background_color: u32,
    pub billboard: u8,
    pub line_width: i32,
    pub see_through: bool,
    pub text_opacity: u8,
}

pub struct TextDisplayStore {
    displays: HashMap<i32, TextDisplayEntity>,
}

impl TextDisplayStore {
    pub fn new() -> Self {
        Self {
            displays: HashMap::new(),
        }
    }

    pub fn spawn(&mut self, id: i32, uuid: uuid::Uuid, position: Position) {
        self.displays.insert(
            id,
            TextDisplayEntity {
                id,
                uuid,
                position,
                prev_position: position,
                text: Vec::new(),
                background_color: 0x40000000,
                billboard: 3,
                line_width: 200,
                see_through: false,
                text_opacity: 255,
            },
        );
    }

    pub fn remove(&mut self, ids: &[i32]) {
        for id in ids {
            self.displays.remove(id);
        }
    }

    pub fn move_delta(&mut self, id: i32, dx: f64, dy: f64, dz: f64) {
        if let Some(display) = self.displays.get_mut(&id) {
            display.prev_position = display.position;
            display.position += glam::dvec3(dx, dy, dz);
        }
    }

    pub fn teleport(&mut self, id: i32, position: Position) {
        if let Some(display) = self.displays.get_mut(&id) {
            display.prev_position = position;
            display.position = position;
        }
    }

    pub fn set_text(&mut self, id: i32, spans: Vec<TextSpan>) {
        if let Some(display) = self.displays.get_mut(&id) {
            display.text = spans;
        }
    }

    pub fn apply_entity_data(&mut self, id: i32, index: u8, value: MetaValue) {
        let Some(display) = self.displays.get_mut(&id) else {
            return;
        };
        match (index, value) {
            (15, MetaValue::Byte(b)) => display.billboard = b,
            (24, MetaValue::Int(w)) => display.line_width = w,
            (25, MetaValue::Int(c)) => display.background_color = c as u32,
            (26, MetaValue::Byte(o)) => display.text_opacity = o,
            (27, MetaValue::Byte(f)) => display.see_through = f & 2 != 0,
            _ => {}
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &TextDisplayEntity> {
        self.displays.values()
    }
}

pub fn build_text_displays(
    elements: &mut Vec<MenuElement>,
    store: &TextDisplayStore,
    partial_tick: f32,
    gs: f32,
    camera_pos: DVec3,
    camera_fov_deg: f32,
    screen_height: f32,
    project: &dyn Fn(DVec3) -> Option<(f32, f32)>,
    spans_width: &dyn Fn(&[TextSpan], f32) -> f32,
) {
    let fov = (camera_fov_deg.to_radians() * 0.5).tan();
    for display in store.iter() {
        if display.text.is_empty() {
            continue;
        }
        let pos = display
            .prev_position
            .lerp(display.position, partial_tick as f64);
        let to_cam = *pos - camera_pos;
        let distance_sq = to_cam.length_squared();
        if distance_sq > 64.0 * 64.0 {
            continue;
        }
        let distance = (distance_sq.sqrt() as f32).max(0.1);
        let Some((screen_x, screen_y)) = project(*pos) else {
            continue;
        };

        let font_scale = (0.25 * screen_height / (2.0 * fov * distance)).clamp(6.0, 32.0 * gs);
        let line_height = font_scale * 1.15;

        let mut lines: Vec<Vec<TextSpan>> = Vec::new();
        let mut current_line = Vec::new();
        for span in &display.text {
            if !span.text.contains('\n') {
                current_line.push(span.clone());
            } else {
                let parts: Vec<&str> = span.text.split('\n').collect();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        lines.push(std::mem::take(&mut current_line));
                    }
                    if !part.is_empty() {
                        let mut s = span.clone();
                        s.text = part.to_string();
                        current_line.push(s);
                    }
                }
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        let total_h = lines.len() as f32 * line_height;
        let start_y = screen_y - total_h * 0.5;

        let bg_a = ((display.background_color >> 24) & 0xff) as f32 / 255.0;
        let bg_r = ((display.background_color >> 16) & 0xff) as f32 / 255.0;
        let bg_g = ((display.background_color >> 8) & 0xff) as f32 / 255.0;
        let bg_b = (display.background_color & 0xff) as f32 / 255.0;

        for (idx, line) in lines.iter().enumerate() {
            let y = start_y + idx as f32 * line_height;
            if bg_a > 0.0 {
                let w = spans_width(line, font_scale);
                if w > 0.0 {
                    let pad = 2.0 * (font_scale / 16.0).max(1.0);
                    elements.push(MenuElement::Rect {
                        x: screen_x - w * 0.5 - pad,
                        y: y - pad,
                        w: w + pad * 2.0,
                        h: font_scale + pad * 2.0,
                        corner_radius: 0.0,
                        color: [bg_r, bg_g, bg_b, bg_a],
                    });
                }
            }
            elements.push(MenuElement::TextSpans {
                x: screen_x,
                y,
                spans: line.clone(),
                scale: font_scale,
                centered: true,
            });
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_display_lifecycle() {
        let mut store = TextDisplayStore::new();
        let uuid = uuid::Uuid::new_v4();
        let pos = Position::new(10.0, 64.0, 20.0);
        store.spawn(100, uuid, pos);
        assert_eq!(store.iter().count(), 1);

        store.move_delta(100, 1.0, 0.0, -1.0);
        let display = store.iter().find(|d| d.id == 100).unwrap();
        assert_eq!(display.position, Position::new(11.0, 64.0, 19.0));

        store.set_text(100, vec![TextSpan::new("Test Hologram".into(), [1.0; 4])]);
        store.apply_entity_data(100, 25, MetaValue::Int(0x80ff0000u32 as i32));
        let display = store.iter().find(|d| d.id == 100).unwrap();
        assert_eq!(display.text.len(), 1);
        assert_eq!(display.background_color, 0x80ff0000);

        store.remove(&[100]);
        assert_eq!(store.iter().count(), 0);
    }

    #[test]
    fn test_build_text_displays_multiline_projection() {
        let mut store = TextDisplayStore::new();
        let uuid = uuid::Uuid::new_v4();
        let pos = Position::new(0.0, 0.0, 5.0);
        store.spawn(1, uuid, pos);
        store.set_text(
            1,
            vec![
                TextSpan::new("Line 1\nLine 2".into(), [1.0; 4]),
            ],
        );

        let mut elements = Vec::new();
        let camera_pos = DVec3::ZERO;
        build_text_displays(
            &mut elements,
            &store,
            0.0,
            1.0,
            camera_pos,
            70.0,
            1080.0,
            &|_p| Some((500.0, 300.0)),
            &|_spans, _scale| 100.0,
        );

        assert_eq!(elements.len(), 4);
    }
}
