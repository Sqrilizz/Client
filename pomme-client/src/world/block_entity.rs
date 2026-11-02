use std::collections::HashMap;

use azalea_registry::builtin::BlockEntityKind;
use simdnbt::owned::NbtCompound;

use crate::position::BlockPos;
use crate::renderer::chunk::block_lighter::LightLevels;
use crate::world::block::BlockStateData;
#[derive(Clone)]
pub struct StoredBlockEntity {
    pub kind: BlockEntityKind,
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SignSideData {
    pub messages: [String; 4],
    pub color_rgb: u32,
    pub glowing: bool,
}

impl SignSideData {
    pub fn is_empty(&self) -> bool {
        self.messages.iter().all(String::is_empty)
    }
}

#[derive(Clone, Debug)]
pub struct SignRenderInfo {
    pub pos: BlockPos,
    pub is_wall: bool,
    pub is_hanging: bool,
    pub angle: f32,
    pub front: SignSideData,
    pub back: SignSideData,
    pub light: LightLevels,
}

pub fn dye_color_rgb(name: &str) -> Option<u32> {
    match name {
        "white" => Some(0xFFFFFF),
        "orange" => Some(0xFF681F),
        "magenta" => Some(0xFF00FF),
        "light_blue" => Some(0x9AC0CD),
        "yellow" => Some(0xFFFF00),
        "lime" => Some(0xBFFF00),
        "pink" => Some(0xFF69B4),
        "gray" => Some(0x808080),
        "light_gray" => Some(0xD3D3D3),
        "cyan" => Some(0x00FFFF),
        "purple" => Some(0xA020F0),
        "blue" => Some(0x0000FF),
        "brown" => Some(0x8B4513),
        "green" => Some(0x00FF00),
        "red" => Some(0xFF0000),
        "black" => Some(0x000000),
        _ => None,
    }
}

pub fn parse_component_text(raw: &str) -> String {
    if raw.is_empty() || raw == "\"\"" {
        return String::new();
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
        match value {
            serde_json::Value::String(s) => return s,
            serde_json::Value::Object(map) => {
                let mut out = String::new();
                if let Some(serde_json::Value::String(t)) = map.get("text") {
                    out.push_str(t);
                }
                if let Some(serde_json::Value::Array(extra)) = map.get("extra") {
                    for item in extra {
                        if let Some(t) = item.as_str() {
                            out.push_str(t);
                        } else if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                            out.push_str(t);
                        }
                    }
                }
                return out;
            }
            _ => {}
        }
    }
    if raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2 {
        raw[1..raw.len() - 1].to_string()
    } else {
        raw.to_string()
    }
}

pub fn parse_sign_render_info(
    pos: BlockPos,
    kind: BlockEntityKind,
    nbt: &NbtCompound,
    block_id: &str,
    props: &crate::world::block::PropMap,
    light: LightLevels,
) -> Option<SignRenderInfo> {
    if !matches!(kind, BlockEntityKind::Sign | BlockEntityKind::HangingSign) {
        return None;
    }
    let is_wall = block_id.ends_with("_wall_sign") || block_id.ends_with("_wall_hanging_sign");
    let is_hanging = block_id.contains("hanging_sign");
    let angle = if is_wall {
        let facing = props.get("facing").unwrap_or("north");
        match facing {
            "south" => 0.0,
            "west" => 90.0,
            "north" => 180.0,
            "east" => 270.0,
            _ => 0.0,
        }
    } else {
        props
            .get("rotation")
            .and_then(|r| r.parse::<f32>().ok())
            .unwrap_or(0.0)
            * 22.5
    };

    let parse_side = |tag: Option<&NbtCompound>| -> SignSideData {
        let mut messages = [String::new(), String::new(), String::new(), String::new()];
        let mut color_rgb = 0u32;
        let mut glowing = false;
        if let Some(t) = tag {
            if let Some(color_name) = t.string("color") {
                if let Some(rgb) = dye_color_rgb(&color_name.to_str()) {
                    color_rgb = rgb;
                }
            }
            if let Some(b) = t.byte("has_glowing_text") {
                glowing = b != 0;
            }
            if let Some(list) = t.list("messages") {
                if let Some(strings) = list.strings() {
                    for (i, s) in strings.iter().enumerate().take(4) {
                        messages[i] = parse_component_text(&s.to_str());
                    }
                } else if let Some(compounds) = list.compounds() {
                    for (i, c) in compounds.into_iter().enumerate().take(4) {
                        if let Some(text) = c.string("text") {
                            messages[i] = text.to_str().into_owned();
                        }
                    }
                }
            }
        }
        SignSideData {
            messages,
            color_rgb,
            glowing,
        }
    };

    let (front, back) = if let Some(front_compound) = nbt.compound("front_text") {
        let back_compound = nbt.compound("back_text");
        (parse_side(Some(front_compound)), parse_side(back_compound))
    } else {
        let mut messages = [String::new(), String::new(), String::new(), String::new()];
        for (i, key) in ["Text1", "Text2", "Text3", "Text4"].iter().enumerate() {
            if let Some(s) = nbt.string(key) {
                messages[i] = parse_component_text(&s.to_str());
            }
        }
        let color_rgb = nbt
            .string("Color")
            .and_then(|c| dye_color_rgb(&c.to_str()))
            .unwrap_or(0);
        let glowing = nbt.byte("GlowingText").is_some_and(|b| b != 0);
        (
            SignSideData {
                messages,
                color_rgb,
                glowing,
            },
            SignSideData::default(),
        )
    };

    if front.is_empty() && back.is_empty() {
        return None;
    }

    Some(SignRenderInfo {
        pos,
        is_wall,
        is_hanging,
        angle,
        front,
        back,
        light,
    })
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
    fn test_parse_component_text() {
        assert_eq!(parse_component_text("{\"text\":\"Leaderboard\"}"), "Leaderboard");
        assert_eq!(parse_component_text("\"Hello\""), "Hello");
        assert_eq!(parse_component_text("\"\""), "");
    }
}
