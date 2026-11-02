use std::collections::HashMap;

use azalea_registry::builtin::BlockEntityKind;
use glam::DVec3;
use simdnbt::owned::NbtCompound;

use crate::position::BlockPos;
use crate::renderer::pipelines::menu_overlay::MenuElement;
use crate::ui::text::TextSpan;
use crate::world::block::{BlockStateData, BlockStateTable};
use crate::world::chunk::ChunkStore;

#[derive(Clone)]
pub struct StoredBlockEntity {
    #[allow(dead_code)]
    pub kind: BlockEntityKind,
    #[allow(dead_code)]
    pub nbt: NbtCompound,
}

pub fn rendered_kind(name: &str) -> Option<BlockEntityKind> {
    match name {
        "chest" => Some(BlockEntityKind::Chest),
        "trapped_chest" => Some(BlockEntityKind::TrappedChest),
        "ender_chest" => Some(BlockEntityKind::EnderChest),
        s if s.ends_with("copper_chest") => Some(BlockEntityKind::Chest),
        s if s == "shulker_box" || s.ends_with("_shulker_box") => Some(BlockEntityKind::ShulkerBox),
        _ => None,
    }
}

fn is_rendered(kind: BlockEntityKind) -> bool {
    matches!(
        kind,
        BlockEntityKind::Chest
            | BlockEntityKind::TrappedChest
            | BlockEntityKind::EnderChest
            | BlockEntityKind::ShulkerBox
    )
}

pub fn sync_block_entity(
    map: &mut HashMap<BlockPos, StoredBlockEntity>,
    pos: BlockPos,
    block_state_data: &BlockStateData,
) {
    let id = block_state_data.id();
    if let Some(kind) = rendered_kind(id) {
        if map.get(&pos).is_none_or(|e| e.kind != kind) {
            map.insert(
                pos,
                StoredBlockEntity {
                    kind,
                    nbt: NbtCompound::default(),
                },
            );
        }
    } else if !is_block_entity_block(id) || map.get(&pos).is_some_and(|e| is_rendered(e.kind)) {
        map.remove(&pos);
    }
}

pub fn is_block_entity_block(name: &str) -> bool {
    rendered_kind(name).is_some()
        || matches!(name, |"oak_sign"| "spruce_sign"
            | "birch_sign"
            | "jungle_sign"
            | "acacia_sign"
            | "dark_oak_sign"
            | "mangrove_sign"
            | "cherry_sign"
            | "pale_oak_sign"
            | "bamboo_sign"
            | "crimson_sign"
            | "warped_sign"
            | "oak_wall_sign"
            | "spruce_wall_sign"
            | "birch_wall_sign"
            | "jungle_wall_sign"
            | "acacia_wall_sign"
            | "dark_oak_wall_sign"
            | "mangrove_wall_sign"
            | "cherry_wall_sign"
            | "pale_oak_wall_sign"
            | "bamboo_wall_sign"
            | "crimson_wall_sign"
            | "warped_wall_sign"
            | "oak_hanging_sign"
            | "spruce_hanging_sign"
            | "birch_hanging_sign"
            | "jungle_hanging_sign"
            | "acacia_hanging_sign"
            | "dark_oak_hanging_sign"
            | "mangrove_hanging_sign"
            | "cherry_hanging_sign"
            | "pale_oak_hanging_sign"
            | "bamboo_hanging_sign"
            | "crimson_hanging_sign"
            | "warped_hanging_sign"
            | "oak_wall_hanging_sign"
            | "spruce_wall_hanging_sign"
            | "birch_wall_hanging_sign"
            | "jungle_wall_hanging_sign"
            | "acacia_wall_hanging_sign"
            | "dark_oak_wall_hanging_sign"
            | "mangrove_wall_hanging_sign"
            | "cherry_wall_hanging_sign"
            | "pale_oak_wall_hanging_sign"
            | "bamboo_wall_hanging_sign"
            | "crimson_wall_hanging_sign"
            | "warped_wall_hanging_sign"
            | "white_banner"
            | "orange_banner"
            | "magenta_banner"
            | "light_blue_banner"
            | "yellow_banner"
            | "lime_banner"
            | "pink_banner"
            | "gray_banner"
            | "light_gray_banner"
            | "cyan_banner"
            | "purple_banner"
            | "blue_banner"
            | "brown_banner"
            | "green_banner"
            | "red_banner"
            | "black_banner"
            | "white_wall_banner"
            | "orange_wall_banner"
            | "magenta_wall_banner"
            | "light_blue_wall_banner"
            | "yellow_wall_banner"
            | "lime_wall_banner"
            | "pink_wall_banner"
            | "gray_wall_banner"
            | "light_gray_wall_banner"
            | "cyan_wall_banner"
            | "purple_wall_banner"
            | "blue_wall_banner"
            | "brown_wall_banner"
            | "green_wall_banner"
            | "red_wall_banner"
            | "black_wall_banner"
            | "skeleton_skull"
            | "skeleton_wall_skull"
            | "wither_skeleton_skull"
            | "wither_skeleton_wall_skull"
            | "zombie_head"
            | "zombie_wall_head"
            | "player_head"
            | "player_wall_head"
            | "creeper_head"
            | "creeper_wall_head"
            | "dragon_head"
            | "dragon_wall_head"
            | "piglin_head"
            | "piglin_wall_head"
            | "conduit"
            | "decorated_pot"
            | "end_portal"
            | "end_gateway"
            | "beacon"
            | "spawner"
            | "trial_spawner"
            | "vault"
            | "brewing_stand"
            | "lectern"
            | "campfire"
            | "soul_campfire"
            | "beehive"
            | "bee_nest"
            | "bell"
            | "suspicious_sand"
            | "suspicious_gravel"
            | "crafter")
}

pub fn is_invisible_block(name: &str) -> bool {
    matches!(
        name,
        "air"
            | "cave_air"
            | "void_air"
            | "barrier"
            | "light"
            | "structure_void"
            | "moving_piston"
            | "water"
            | "lava"
            | "bubble_column"
            | "end_portal"
            | "end_gateway"
    )
}

pub fn is_fluid_block(name: &str) -> bool {
    matches!(name, "water" | "lava" | "bubble_column")
}

fn parse_sign_line(raw: &str, default_color: [f32; 4]) -> Vec<TextSpan> {
    if raw.is_empty() || raw == "\"\"" {
        return Vec::new();
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Ok(comp) = crate::chat_component::Component::from_value(&value) {
            let spans = crate::ui::text::format_component_spans(&comp, default_color);
            if !spans.is_empty() && spans.iter().any(|s| !s.text.is_empty()) {
                return spans;
            }
        }
    }
    let text = if raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2 {
        &raw[1..raw.len() - 1]
    } else {
        raw
    };
    if text.is_empty() {
        Vec::new()
    } else {
        vec![TextSpan::new(text.to_string(), default_color)]
    }
}

fn sign_dye_color(name: Option<&str>) -> [f32; 4] {
    match name {
        Some("white") => [0.976, 1.0, 0.996, 1.0],
        Some("orange") => [0.976, 0.537, 0.110, 1.0],
        Some("magenta") => [0.780, 0.282, 0.737, 1.0],
        Some("light_blue") => [0.400, 0.698, 0.898, 1.0],
        Some("yellow") => [0.996, 0.851, 0.243, 1.0],
        Some("lime") => [0.502, 0.776, 0.122, 1.0],
        Some("pink") => [0.953, 0.553, 0.671, 1.0],
        Some("gray") => [0.278, 0.314, 0.322, 1.0],
        Some("light_gray") => [0.620, 0.620, 0.592, 1.0],
        Some("cyan") => [0.086, 0.490, 0.529, 1.0],
        Some("purple") => [0.533, 0.200, 0.710, 1.0],
        Some("blue") => [0.235, 0.286, 0.686, 1.0],
        Some("brown") => [0.514, 0.325, 0.200, 1.0],
        Some("green") => [0.369, 0.506, 0.157, 1.0],
        Some("red") => [0.690, 0.180, 0.149, 1.0],
        _ => [0.0, 0.0, 0.0, 1.0],
    }
}

pub fn build_sign_texts(
    elements: &mut Vec<MenuElement>,
    chunk_store: &ChunkStore,
    block_state_table: BlockStateTable,
    camera_pos: DVec3,
    camera_fov_deg: f32,
    screen_height: f32,
    gs: f32,
    project: &dyn Fn(DVec3) -> Option<(f32, f32)>,
) {
    let fov = (camera_fov_deg.to_radians() * 0.5).tan();
    for (pos, entry) in &chunk_store.block_entities {
        if !matches!(
            entry.kind,
            BlockEntityKind::Sign | BlockEntityKind::HangingSign
        ) {
            continue;
        }
        let block_center = DVec3::new(pos.x as f64 + 0.5, pos.y as f64 + 0.5, pos.z as f64 + 0.5);
        let dist_sq = (block_center - camera_pos).length_squared();
        if dist_sq > 24.0 * 24.0 {
            continue;
        }
        let state = chunk_store.get_block_state(pos.x, pos.y, pos.z);
        let data = block_state_table.block_state_data(state);
        let id = data.id();

        let (is_front, normal, board_center) = if id.ends_with("_wall_sign") {
            let facing = data.properties().get("facing").unwrap_or("north");
            let (facing_normal, offset_center) = match facing {
                "north" => (DVec3::new(0.0, 0.0, -1.0), DVec3::new(0.5, 0.45, 0.92)),
                "south" => (DVec3::new(0.0, 0.0, 1.0), DVec3::new(0.5, 0.45, 0.08)),
                "west" => (DVec3::new(-1.0, 0.0, 0.0), DVec3::new(0.92, 0.45, 0.5)),
                "east" => (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.08, 0.45, 0.5)),
                _ => (DVec3::new(0.0, 0.0, -1.0), DVec3::new(0.5, 0.45, 0.92)),
            };
            let center = DVec3::new(pos.x as f64, pos.y as f64, pos.z as f64) + offset_center;
            let to_cam = (camera_pos - center).normalize();
            if facing_normal.dot(to_cam) <= 0.0 {
                continue;
            }
            (true, facing_normal, center)
        } else if id.ends_with("_hanging_sign") || id.ends_with("_wall_hanging_sign") {
            let facing = data.properties().get("facing").unwrap_or("north");
            let facing_normal = match facing {
                "north" => DVec3::new(0.0, 0.0, -1.0),
                "south" => DVec3::new(0.0, 0.0, 1.0),
                "west" => DVec3::new(-1.0, 0.0, 0.0),
                "east" => DVec3::new(1.0, 0.0, 0.0),
                _ => DVec3::new(0.0, 0.0, -1.0),
            };
            let center = DVec3::new(pos.x as f64 + 0.5, pos.y as f64 + 0.35, pos.z as f64 + 0.5);
            let to_cam = (camera_pos - center).normalize();
            let dot = facing_normal.dot(to_cam);
            if dot >= 0.0 {
                (true, facing_normal, center)
            } else {
                (false, -facing_normal, center)
            }
        } else {
            let rot: f64 = data
                .properties()
                .get("rotation")
                .and_then(|r| r.parse().ok())
                .unwrap_or(0.0);
            let angle = rot * (std::f64::consts::PI / 8.0);
            let standing_normal = DVec3::new(-angle.sin(), 0.0, angle.cos());
            let center = DVec3::new(pos.x as f64 + 0.5, pos.y as f64 + 0.70, pos.z as f64 + 0.5);
            let to_cam = (camera_pos - center).normalize();
            let dot = standing_normal.dot(to_cam);
            if dot >= 0.0 {
                (true, standing_normal, center)
            } else {
                (false, -standing_normal, center)
            }
        };

        let face_position = board_center + normal * 0.045;
        let delta = face_position - camera_pos;
        let dist = delta.length() as f32;
        if dist < 0.1 || dist > 24.0 {
            continue;
        }

        let dir = (delta / dist as f64).as_vec3();
        let occluded = if let Some(hit) = crate::player::interaction::raycast(
            camera_pos,
            dir,
            dist,
            chunk_store,
            block_state_table,
            None,
        ) {
            (hit.hit_point - camera_pos).length() < (dist - 0.25) as f64
        } else {
            false
        };
        if occluded {
            continue;
        }

        let (messages, color_name) = if is_front {
            if let Some(front) = entry.nbt.compound("front_text") {
                let color = front
                    .string("color")
                    .map(|s| s.to_str().into_owned());
                let mut lines = Vec::new();
                if let Some(list) = front.list("messages") {
                    if let Some(strings) = list.strings() {
                        for s in strings {
                            lines.push(s.to_str().into_owned());
                        }
                    }
                }
                (lines, color)
            } else {
                let mut lines = Vec::new();
                for key in ["Text1", "Text2", "Text3", "Text4"] {
                    if let Some(s) = entry.nbt.string(key) {
                        lines.push(s.to_str().into_owned());
                    } else {
                        lines.push(String::new());
                    }
                }
                let color = entry
                    .nbt
                    .string("Color")
                    .map(|s| s.to_str().into_owned());
                (lines, color)
            }
        } else {
            if let Some(back) = entry.nbt.compound("back_text") {
                let color = back
                    .string("color")
                    .map(|s| s.to_str().into_owned());
                let mut lines = Vec::new();
                if let Some(list) = back.list("messages") {
                    if let Some(strings) = list.strings() {
                        for s in strings {
                            lines.push(s.to_str().into_owned());
                        }
                    }
                }
                (lines, color)
            } else {
                (Vec::new(), None)
            }
        };

        if messages.is_empty() || messages.iter().all(|m| m.is_empty() || m == "\"\"") {
            continue;
        }

        let default_color = sign_dye_color(color_name.as_deref());
        let font_scale = (0.104 * screen_height / (2.0 * fov * dist)).clamp(2.5, 14.0 * gs);
        if font_scale < 2.5 {
            continue;
        }

        for (i, raw) in messages.iter().enumerate().take(4) {
            let spans = parse_sign_line(raw, default_color);
            if spans.is_empty() {
                continue;
            }
            let line_y = face_position.y + (1.5 - i as f64) * 0.105;
            let line_pos = DVec3::new(face_position.x, line_y, face_position.z);
            let Some((screen_x, screen_y)) = project(line_pos) else {
                continue;
            };
            elements.push(MenuElement::TextSpans {
                x: screen_x,
                y: screen_y - font_scale * 0.5,
                spans,
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
    fn native_beds_are_modeled_blocks_not_block_entities() {
        assert!(!is_block_entity_block("red_bed"));
        assert!(!is_block_entity_block("white_bed"));
        assert!(rendered_kind("red_bed").is_none());
    }

    #[test]
    fn test_parse_sign_line() {
        let spans = parse_sign_line("{\"text\":\"Leaderboard\"}", [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "Leaderboard");

        let spans_plain = parse_sign_line("\"Hello\"", [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(spans_plain.len(), 1);
        assert_eq!(spans_plain[0].text, "Hello");

        let spans_empty = parse_sign_line("\"\"", [0.0, 0.0, 0.0, 1.0]);
        assert!(spans_empty.is_empty());
    }
}
