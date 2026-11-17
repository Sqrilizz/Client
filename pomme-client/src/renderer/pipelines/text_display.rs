use glam::{DVec3, Mat4, Quat, Vec3};

use crate::renderer::chunk::block_lighter::LightLevels;
use crate::renderer::pipelines::world_text::{
    MAX_WORLD_TEXT_VERTS, WorldTextVertices, push_quad, push_spans, spans_advance, spans_width,
};
use crate::ui::text::{TextSpan, font_spans};

pub const TEXT_DISPLAY_SCALE: f32 = 0.025;

#[derive(Clone, Debug)]
pub struct TextDisplayRenderInfo {
    pub entity_id: i32,
    pub position: DVec3,
    pub y_rot_deg: f32,
    pub x_rot_deg: f32,
    pub spans: Vec<TextSpan>,
    pub translation: Vec3,
    pub scale: Vec3,
    pub left_rotation: Quat,
    pub right_rotation: Quat,
    pub billboard: u8,
    pub line_width: i32,
    pub background_color: i32,
    pub text_opacity: i8,
    pub style_flags: u8,
    pub light: LightLevels,
}

fn is_pua_or_format(c: char) -> bool {
    c.is_whitespace()
        || c.is_control()
        || ('\u{e000}'..='\u{f8ff}').contains(&c)
        || ('\u{f0000}'..='\u{ffffd}').contains(&c)
        || ('\u{100000}'..='\u{10fffd}').contains(&c)
        || ('\u{200b}'..='\u{200f}').contains(&c)
        || c == '\u{feff}'
}

fn is_foreground_display(display: &TextDisplayRenderInfo) -> bool {
    display
        .spans
        .iter()
        .any(|span| span.inline_object.is_some() || span.text.chars().any(|c| !is_pua_or_format(c)))
}

pub fn push_text_displays(
    verts: &mut WorldTextVertices,
    camera: &crate::renderer::camera::Camera,
    camera_render_pos: DVec3,
    anchor: DVec3,
    ctx: &mut crate::renderer::pipelines::world_text::WorldTextContext<'_>,
    displays: &[TextDisplayRenderInfo],
) {
    let forward = camera.forward();
    let (cam_yaw_deg, cam_pitch_deg) = camera.effective_look_deg();

    let calc_billboard = |display: &TextDisplayRenderInfo| match display.billboard {
        0 => {
            Quat::from_rotation_y((-display.y_rot_deg).to_radians())
                * Quat::from_rotation_x(display.x_rot_deg.to_radians())
        }
        1 => {
            Quat::from_rotation_y((180.0 - cam_yaw_deg).to_radians())
                * Quat::from_rotation_x(display.x_rot_deg.to_radians())
        }
        2 => {
            Quat::from_rotation_y((-display.y_rot_deg).to_radians())
                * Quat::from_rotation_x(cam_pitch_deg.to_radians())
        }
        _ => {
            Quat::from_rotation_y((180.0 - cam_yaw_deg).to_radians())
                * Quat::from_rotation_x(cam_pitch_deg.to_radians())
        }
    };

    let bucket_size = 4.0;
    let dists: Vec<i64> = displays
        .iter()
        .map(|display| {
            let billboard_quat = calc_billboard(display);
            let effective_pos =
                display.position + (billboard_quat * display.translation).as_dvec3();
            let to_cam = effective_pos - camera_render_pos;
            let depth = to_cam.dot(forward.as_dvec3());
            (depth / bucket_size).floor() as i64
        })
        .collect();

    let foreground_flags: Vec<bool> = displays.iter().map(is_foreground_display).collect();
    let mut order: Vec<usize> = (0..displays.len()).collect();
    order.sort_by(|&a, &b| {
        dists[b]
            .cmp(&dists[a])
            .then_with(|| foreground_flags[a].cmp(&foreground_flags[b]))
            .then_with(|| {
                displays[a]
                    .translation
                    .z
                    .total_cmp(&displays[b].translation.z)
            })
            .then_with(|| displays[a].entity_id.cmp(&displays[b].entity_id))
    });

    for &idx in &order {
        let display = &displays[idx];
        if verts.len() + 256 > MAX_WORLD_TEXT_VERTS {
            break;
        }
        let billboard_quat = calc_billboard(display);
        let effective_pos = display.position + (billboard_quat * display.translation).as_dvec3();
        let to_cam = (camera_render_pos - effective_pos).as_vec3();
        if to_cam.dot(forward) >= 0.0 {
            continue;
        }

        let line_limit = if display.line_width > 0 {
            display.line_width as f32
        } else {
            f32::MAX
        };

        let width_fn = |s: &[TextSpan]| spans_advance(s, ctx.glyph_map);
        let mut wrapped_lines: Vec<Vec<TextSpan>> = Vec::new();
        for para in split_spans_by_newline(&display.spans) {
            let font_para = font_spans(&para);
            if line_limit < f32::MAX {
                let wrapped = crate::ui::chat::wrap_spans(&font_para, line_limit, &width_fn);
                for line in wrapped {
                    wrapped_lines.push(line);
                }
            } else {
                wrapped_lines.push(font_para.into_owned());
            }
        }
        if wrapped_lines.is_empty() {
            continue;
        }

        let measured_lines: Vec<(Vec<TextSpan>, f32)> = wrapped_lines
            .into_iter()
            .map(|l| {
                let w = spans_width(&l, ctx.glyph_map);
                (l, w)
            })
            .collect();

        let max_w = measured_lines
            .iter()
            .map(|(_, w)| *w)
            .fold(0.0f32, f32::max);
        if max_w <= 0.0 {
            continue;
        }

        let line_count = measured_lines.len() as f32;
        let total_h = (line_count * 10.0 - 1.0).max(0.0);

        let entity_pos = (display.position - anchor).as_vec3();
        let base_mat = Mat4::from_translation(entity_pos)
            * Mat4::from_quat(billboard_quat)
            * Mat4::from_translation(display.translation)
            * Mat4::from_quat(display.left_rotation)
            * Mat4::from_scale(display.scale)
            * Mat4::from_quat(display.right_rotation)
            * Mat4::from_rotation_y(std::f32::consts::PI)
            * Mat4::from_scale(Vec3::splat(-TEXT_DISPLAY_SCALE))
            * Mat4::from_translation(Vec3::new(1.0 - max_w / 2.0, -total_h, 0.0));

        let see_through = (display.style_flags & 0x02) != 0;
        let target_verts = if see_through {
            &mut verts.see_through
        } else {
            &mut verts.depth_tested
        };

        let light = display.light.lane();

        let use_default_bg = (display.style_flags & 0x04) != 0;
        let bg_color = if use_default_bg {
            Some([0.0, 0.0, 0.0, 0.25])
        } else if display.background_color != 0 {
            let a = ((display.background_color >> 24) & 0xff) as f32 / 255.0;
            let r = ((display.background_color >> 16) & 0xff) as f32 / 255.0;
            let g = ((display.background_color >> 8) & 0xff) as f32 / 255.0;
            let b = (display.background_color & 0xff) as f32 / 255.0;
            if a > 0.0 { Some([r, g, b, a]) } else { None }
        } else {
            None
        };

        let alignment = (display.style_flags >> 3) & 0x03;
        let opacity = if display.text_opacity != -1 {
            (display.text_opacity as u8 as f32) / 255.0
        } else {
            1.0
        };

        for (i, (line, line_w)) in measured_lines.iter().enumerate() {
            let line_offset = match alignment {
                1 => 0.0,
                2 => max_w - line_w,
                _ => (max_w - line_w) / 2.0,
            };
            let y_offset = i as f32 * 10.0;
            let at_line =
                |x: f32, y: f32| base_mat.transform_point3(Vec3::new(x, y_offset + y, 0.0));

            if let Some(mut bg) = bg_color {
                bg[3] *= opacity;
                let bg_corners = [
                    base_mat.transform_point3(Vec3::new(line_offset - 1.0, y_offset - 1.0, 0.0)),
                    base_mat.transform_point3(Vec3::new(
                        line_offset + line_w + 1.0,
                        y_offset - 1.0,
                        0.0,
                    )),
                    base_mat.transform_point3(Vec3::new(
                        line_offset + line_w + 1.0,
                        y_offset + 9.0,
                        0.0,
                    )),
                    base_mat.transform_point3(Vec3::new(line_offset - 1.0, y_offset + 9.0, 0.0)),
                ];
                push_quad(target_verts, bg_corners, [0.0; 4], 0.0, -1.0, bg, light);
            }

            if (display.style_flags & 0x01) != 0 {
                let shadow_color = |span: &TextSpan| {
                    let mut c = span.color;
                    c[0] *= 0.25;
                    c[1] *= 0.25;
                    c[2] *= 0.25;
                    c[3] *= opacity;
                    c
                };
                let at_shadow = |x: f32, y: f32| {
                    base_mat.transform_point3(Vec3::new(x + 1.0, y_offset + y + 1.0, 0.0))
                };
                let shadow_obj = |_: &TextSpan| [0.25, 0.25, 0.25, opacity];
                push_spans(
                    target_verts,
                    ctx,
                    line,
                    line_offset,
                    &at_shadow,
                    &shadow_color,
                    Some(&shadow_obj),
                    light,
                );
            }

            let text_color = |span: &TextSpan| {
                let mut c = span.color;
                c[3] *= opacity;
                c
            };
            let text_obj = |_: &TextSpan| [1.0, 1.0, 1.0, opacity];
            push_spans(
                target_verts,
                ctx,
                line,
                line_offset,
                &at_line,
                &text_color,
                Some(&text_obj),
                light,
            );
        }
    }
}

pub fn split_spans_by_newline(spans: &[TextSpan]) -> Vec<Vec<TextSpan>> {
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    for span in spans {
        if !span.text.contains('\n') {
            current_line.push(span.clone());
            continue;
        }
        let parts: Vec<&str> = span.text.split('\n').collect();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                lines.push(std::mem::take(&mut current_line));
            }
            if !part.is_empty() {
                let mut s = span.clone();
                s.text = (*part).to_string();
                current_line.push(s);
            }
        }
    }
    if !current_line.is_empty() || lines.is_empty() {
        lines.push(current_line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_spans_handles_newlines() {
        let span1 = TextSpan::new("line1\nline2".to_string(), [1.0; 4]);
        let lines = split_spans_by_newline(&[span1]);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0][0].text, "line1");
        assert_eq!(lines[1][0].text, "line2");
    }

    #[test]
    fn split_spans_handles_multiple_newlines() {
        let span1 = TextSpan::new("a\n\nb".to_string(), [1.0; 4]);
        let lines = split_spans_by_newline(&[span1]);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0][0].text, "a");
        assert!(lines[1].is_empty());
        assert_eq!(lines[2][0].text, "b");
    }

    #[test]
    fn display_store_applies_translation_and_scale() {
        let mut store = crate::entity::display::DisplayEntityStore::new();
        store.spawn_text_display(42, crate::entity::Position::new(10.0, 20.0, 30.0), 0.0, 0.0);
        store.apply_data(42, 11, &crate::entity::MetaValue::Vector3([0.0, 2.5, 0.0]));
        store.apply_data(42, 12, &crate::entity::MetaValue::Vector3([1.5, 1.5, 1.5]));
        store.apply_data(42, 25, &crate::entity::MetaValue::Int(0));

        let display = store.text_displays.get(&42).unwrap();
        assert_eq!(display.translation, Vec3::new(0.0, 2.5, 0.0));
        assert_eq!(display.scale, Vec3::new(1.5, 1.5, 1.5));
        assert_eq!(display.background_color, 0);
    }

    #[test]
    fn display_store_applies_view_range() {
        let mut store = crate::entity::display::DisplayEntityStore::new();
        store.spawn_text_display(42, crate::entity::Position::new(10.0, 20.0, 30.0), 0.0, 0.0);
        let display = store.text_displays.get(&42).unwrap();
        assert_eq!(display.view_range, 1.0);
        store.apply_data(42, 17, &crate::entity::MetaValue::Float(0.5));
        let display = store.text_displays.get(&42).unwrap();
        assert_eq!(display.view_range, 0.5);
    }

    #[test]
    fn inline_object_advance_matches_spec() {
        let mut span = TextSpan::new("\u{fffc}".to_string(), [1.0; 4]);
        span.inline_object = Some(crate::ui::text::InlineObject::Player {
            uuid: None,
            name: Some("test".into()),
            textures: None,
            hat: true,
        });
        let gm = crate::ui::font::GlyphMap::empty();
        assert_eq!(spans_advance(&[span.clone()], &gm), 8.0);
        span.bold = true;
        assert_eq!(spans_advance(&[span], &gm), 9.0);
    }

    #[test]
    fn text_display_order_satisfies_total_order() {
        let mut displays = Vec::new();
        for id in 0..50 {
            displays.push(TextDisplayRenderInfo {
                entity_id: id,
                position: DVec3::new(id as f64 * 0.001, 0.0, 10.0),
                y_rot_deg: 0.0,
                x_rot_deg: 0.0,
                spans: vec![TextSpan::new("test".into(), [1.0; 4])],
                translation: Vec3::ZERO,
                scale: Vec3::ONE,
                left_rotation: Quat::IDENTITY,
                right_rotation: Quat::IDENTITY,
                billboard: 0,
                line_width: 200,
                background_color: 0,
                text_opacity: -1,
                style_flags: 0,
                light: LightLevels::new(15, 15),
            });
        }
        let forward = Vec3::Z;
        let camera_render_pos = DVec3::ZERO;
        let bucket_size = 4.0;
        let dists: Vec<i64> = displays
            .iter()
            .map(|d| {
                let to_cam = d.position - camera_render_pos;
                let depth = to_cam.dot(forward.as_dvec3());
                (depth / bucket_size).floor() as i64
            })
            .collect();
        let foreground_flags: Vec<bool> = displays.iter().map(is_foreground_display).collect();
        let mut order: Vec<usize> = (0..displays.len()).collect();
        order.sort_by(|&a, &b| {
            dists[b]
                .cmp(&dists[a])
                .then_with(|| foreground_flags[a].cmp(&foreground_flags[b]))
                .then_with(|| {
                    displays[a]
                        .translation
                        .z
                        .total_cmp(&displays[b].translation.z)
                })
                .then_with(|| displays[a].entity_id.cmp(&displays[b].entity_id))
        });
        assert_eq!(order.len(), 50);
    }

    #[test]
    fn background_displays_sort_before_foreground() {
        let displays = vec![
            TextDisplayRenderInfo {
                entity_id: 10,
                position: DVec3::new(0.0, 0.0, 10.0),
                y_rot_deg: 0.0,
                x_rot_deg: 0.0,
                spans: vec![TextSpan::new("#1".into(), [1.0; 4])],
                translation: Vec3::ZERO,
                scale: Vec3::ONE,
                left_rotation: Quat::IDENTITY,
                right_rotation: Quat::IDENTITY,
                billboard: 0,
                line_width: 200,
                background_color: 0,
                text_opacity: -1,
                style_flags: 0,
                light: LightLevels::new(15, 15),
            },
            TextDisplayRenderInfo {
                entity_id: 20,
                position: DVec3::new(0.0, 0.0, 10.0),
                y_rot_deg: 0.0,
                x_rot_deg: 0.0,
                spans: vec![TextSpan::new("\u{eb40}".into(), [1.0; 4])],
                translation: Vec3::ZERO,
                scale: Vec3::ONE,
                left_rotation: Quat::IDENTITY,
                right_rotation: Quat::IDENTITY,
                billboard: 0,
                line_width: 200,
                background_color: 0,
                text_opacity: -1,
                style_flags: 0,
                light: LightLevels::new(15, 15),
            },
        ];
        let forward = Vec3::Z;
        let camera_render_pos = DVec3::ZERO;
        let bucket_size = 4.0;
        let dists: Vec<i64> = displays
            .iter()
            .map(|d| {
                let to_cam = d.position - camera_render_pos;
                let depth = to_cam.dot(forward.as_dvec3());
                (depth / bucket_size).floor() as i64
            })
            .collect();
        let foreground_flags: Vec<bool> = displays.iter().map(is_foreground_display).collect();
        let mut order: Vec<usize> = (0..displays.len()).collect();
        order.sort_by(|&a, &b| {
            dists[b]
                .cmp(&dists[a])
                .then_with(|| foreground_flags[a].cmp(&foreground_flags[b]))
                .then_with(|| {
                    displays[a]
                        .translation
                        .z
                        .total_cmp(&displays[b].translation.z)
                })
                .then_with(|| displays[a].entity_id.cmp(&displays[b].entity_id))
        });
        assert_eq!(order, vec![1, 0]);
    }
}
