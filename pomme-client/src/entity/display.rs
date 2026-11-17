use std::collections::HashMap;

use glam::{DVec3, Quat, Vec3};

use crate::entity::dimensions::EntityDimensions;
use crate::entity::{MetaValue, Position};
use crate::player::interaction::EntityHitResult;
use crate::ui::text::TextSpan;

pub struct TextDisplayEntity {
    pub position: Position,
    pub prev_position: Position,
    pub y_rot_deg: f32,
    pub x_rot_deg: f32,
    pub text: Option<String>,
    pub spans: Option<Vec<TextSpan>>,
    pub translation: Vec3,
    pub scale: Vec3,
    pub left_rotation: Quat,
    pub right_rotation: Quat,
    pub billboard: u8,
    pub brightness: i32,
    pub view_range: f32,
    pub line_width: i32,
    pub background_color: i32,
    pub text_opacity: i8,
    pub style_flags: u8,
}

pub struct InteractionEntity {
    pub position: Position,
    pub prev_position: Position,
    pub width: f32,
    pub height: f32,
    pub response: bool,
}

#[derive(Default)]
pub struct DisplayEntityStore {
    pub text_displays: HashMap<i32, TextDisplayEntity>,
    pub interactions: HashMap<i32, InteractionEntity>,
}

impl DisplayEntityStore {
    pub fn new() -> Self {
        Self {
            text_displays: HashMap::new(),
            interactions: HashMap::new(),
        }
    }

    pub fn spawn_text_display(
        &mut self,
        id: i32,
        position: Position,
        y_rot_deg: f32,
        x_rot_deg: f32,
    ) {
        self.text_displays.insert(
            id,
            TextDisplayEntity {
                position,
                prev_position: position,
                y_rot_deg,
                x_rot_deg,
                text: None,
                spans: None,
                translation: Vec3::ZERO,
                scale: Vec3::ONE,
                left_rotation: Quat::IDENTITY,
                right_rotation: Quat::IDENTITY,
                billboard: 0,
                brightness: -1,
                view_range: 1.0,
                line_width: 200,
                background_color: 0x40000000,
                text_opacity: -1,
                style_flags: 0,
            },
        );
    }

    pub fn spawn_interaction(&mut self, id: i32, position: Position) {
        self.interactions.insert(
            id,
            InteractionEntity {
                position,
                prev_position: position,
                width: 1.0,
                height: 1.0,
                response: false,
            },
        );
    }

    pub fn move_delta(&mut self, id: i32, dx: f64, dy: f64, dz: f64) {
        if let Some(display) = self.text_displays.get_mut(&id) {
            display.prev_position = display.position;
            display.position += DVec3::new(dx, dy, dz);
        }
        if let Some(interaction) = self.interactions.get_mut(&id) {
            interaction.prev_position = interaction.position;
            interaction.position += DVec3::new(dx, dy, dz);
        }
    }

    pub fn rotate(&mut self, id: i32, y_rot_deg: f32, x_rot_deg: f32) {
        if let Some(display) = self.text_displays.get_mut(&id) {
            display.y_rot_deg = y_rot_deg;
            display.x_rot_deg = x_rot_deg;
        }
    }

    pub fn teleport(&mut self, id: i32, position: Position) {
        if let Some(display) = self.text_displays.get_mut(&id) {
            display.prev_position = position;
            display.position = position;
        }
        if let Some(interaction) = self.interactions.get_mut(&id) {
            interaction.prev_position = position;
            interaction.position = position;
        }
    }

    pub fn remove(&mut self, ids: &[i32]) {
        for id in ids {
            self.text_displays.remove(id);
            self.interactions.remove(id);
        }
    }

    pub fn set_text(&mut self, id: i32, text: Option<String>, spans: Option<Vec<TextSpan>>) {
        if let Some(display) = self.text_displays.get_mut(&id) {
            display.text = text;
            display.spans = spans;
        }
    }

    pub fn set_custom_name(&mut self, id: i32, text: Option<String>, spans: Option<Vec<TextSpan>>) {
        if let Some(display) = self.text_displays.get_mut(&id) {
            if display.spans.is_none() {
                display.text = text;
                display.spans = spans;
            }
        }
    }

    pub fn apply_data(&mut self, id: i32, index: u8, value: &MetaValue) {
        if let Some(display) = self.text_displays.get_mut(&id) {
            match (index, value) {
                (11, MetaValue::Vector3(v)) => display.translation = Vec3::from_slice(v),
                (12, MetaValue::Vector3(v)) => display.scale = Vec3::from_slice(v),
                (13, MetaValue::Quaternion(q)) => {
                    display.left_rotation = Quat::from_xyzw(q[0], q[1], q[2], q[3])
                }
                (14, MetaValue::Quaternion(q)) => {
                    display.right_rotation = Quat::from_xyzw(q[0], q[1], q[2], q[3])
                }
                (15, MetaValue::Byte(b)) => display.billboard = *b,
                (16, MetaValue::Int(b)) => display.brightness = *b,
                (17, MetaValue::Float(r)) => display.view_range = *r,
                (24, MetaValue::Int(w)) => display.line_width = *w,
                (25, MetaValue::Int(c)) => display.background_color = *c,
                (26, MetaValue::Byte(o)) => display.text_opacity = *o as i8,
                (27, MetaValue::Byte(f)) => display.style_flags = *f,
                _ => {}
            }
        }
        if let Some(interaction) = self.interactions.get_mut(&id) {
            match (index, value) {
                (8, MetaValue::Float(w)) => interaction.width = *w,
                (9, MetaValue::Float(h)) => interaction.height = *h,
                (10, MetaValue::Bool(r)) => interaction.response = *r,
                _ => {}
            }
        }
    }

    pub fn nearest_interaction_hit(
        &self,
        from: DVec3,
        to: DVec3,
        mut nearest_dist_sq: f64,
        mut nearest: Option<EntityHitResult>,
    ) -> (f64, Option<EntityHitResult>) {
        for (&entity_id, interaction) in &self.interactions {
            let dims = EntityDimensions::new(interaction.width, interaction.height);
            let aabb = dims.make_bounding_box(*interaction.position);
            if let Some(hit) = aabb.clip(from, to) {
                let dist_sq = from.distance_squared(hit);
                if dist_sq < nearest_dist_sq {
                    nearest_dist_sq = dist_sq;
                    nearest = Some(EntityHitResult {
                        entity_id,
                        location: hit,
                        entity_pos: *interaction.position,
                    });
                }
            }
        }
        (nearest_dist_sq, nearest)
    }
}
