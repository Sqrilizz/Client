use glam::Vec2;

use crate::app::input::touch::{TouchButton, TouchControls, TouchLayout};
use crate::renderer::pipelines::menu_overlay::MenuElement;
use crate::ui::common::{FONT_SIZE, WHITE};

const BG_IDLE: [f32; 4] = [0.08, 0.08, 0.08, 0.50];
const BORDER_IDLE: [f32; 4] = [1.0, 1.0, 1.0, 0.22];
const BG_ACTIVE: [f32; 4] = [0.20, 0.45, 0.85, 0.70];
const BORDER_ACTIVE: [f32; 4] = [1.0, 1.0, 1.0, 0.85];
const CURSOR_BODY: [f32; 4] = [1.0, 1.0, 1.0, 0.95];
const CURSOR_OUTLINE: [f32; 4] = [0.0, 0.0, 0.0, 0.95];

pub fn build_touch_controls(
    elements: &mut Vec<MenuElement>,
    layout: &TouchLayout,
    touch: &TouchControls,
    menu_mode: bool,
) {
    let gs = layout.gs;

    if menu_mode {
        let menu_buttons: &[TouchButton] = if touch.is_virtual_mouse_mode() {
            &[
                TouchButton::Keyboard,
                TouchButton::MouseToggle,
                TouchButton::Primary,
                TouchButton::Secondary,
            ]
        } else {
            &[TouchButton::Keyboard, TouchButton::MouseToggle]
        };

        for &button in menu_buttons {
            let (x, y, w, h) = layout.button_rect(button);
            let active = match button {
                TouchButton::MouseToggle => touch.is_virtual_mouse_mode(),
                TouchButton::Primary => touch.is_primary_held(),
                TouchButton::Secondary => touch.is_secondary_held(),
                _ => false,
            };
            draw_beveled_box(elements, x, y, w, h, 4.0 * gs, active, gs);
            draw_button_text(
                elements,
                x + w / 2.0,
                y + h / 2.0,
                button.label(),
                FONT_SIZE * gs * 0.75,
            );
        }

        if touch.is_virtual_mouse_mode() {
            push_virtual_cursor(elements, touch.virtual_cursor_position(), gs);
        }
        return;
    }

    let dpad = layout.dpad_home();
    let cell = layout.dpad_cell();
    let plate_w = cell;
    let plate_h = 3.0 * cell;
    let plate_color = [0.0, 0.0, 0.0, 0.35];

    elements.push(MenuElement::Rect {
        x: dpad.x - plate_w / 2.0,
        y: dpad.y - plate_h / 2.0,
        w: plate_w,
        h: plate_h,
        corner_radius: 8.0 * gs,
        color: plate_color,
    });
    elements.push(MenuElement::Rect {
        x: dpad.x - plate_h / 2.0,
        y: dpad.y - plate_w / 2.0,
        w: plate_h,
        h: plate_w,
        corner_radius: 8.0 * gs,
        color: plate_color,
    });

    for &btn in &[
        TouchButton::Forward,
        TouchButton::Backward,
        TouchButton::Left,
        TouchButton::Right,
        TouchButton::DpadCenter,
    ] {
        let (x, y, w, h) = layout.button_rect(btn);
        let pad = 2.0 * gs;
        let active = match btn {
            TouchButton::Forward => touch.is_forward_held(),
            TouchButton::Backward => touch.is_backward_held(),
            TouchButton::Left => touch.is_left_held(),
            TouchButton::Right => touch.is_right_held(),
            TouchButton::DpadCenter => touch.is_sneak_locked(),
            _ => false,
        };
        let radius = if btn == TouchButton::DpadCenter {
            3.0 * gs
        } else {
            5.0 * gs
        };
        draw_beveled_box(
            elements,
            x + pad,
            y + pad,
            w - 2.0 * pad,
            h - 2.0 * pad,
            radius,
            active,
            gs,
        );

        let cx = x + w / 2.0;
        let cy = y + h / 2.0;
        let u = 2.0 * gs;
        match btn {
            TouchButton::Forward => draw_pixel_arrow_up(elements, cx, cy, u, WHITE),
            TouchButton::Backward => draw_pixel_arrow_down(elements, cx, cy, u, WHITE),
            TouchButton::Left => draw_pixel_arrow_left(elements, cx, cy, u, WHITE),
            TouchButton::Right => draw_pixel_arrow_right(elements, cx, cy, u, WHITE),
            TouchButton::DpadCenter => draw_pixel_diamond(elements, cx, cy, u, WHITE),
            _ => {}
        }
    }

    let (ix, iy, iw, ih) = layout.button_rect(TouchButton::Inventory);
    draw_beveled_box(elements, ix, iy, iw, ih, 3.0 * gs, false, gs);
    draw_pixel_dots(elements, ix + iw / 2.0, iy + ih / 2.0, 1.5 * gs, WHITE);

    let (jx, jy, jw, jh) = layout.button_rect(TouchButton::Jump);
    let jump_active = touch.jump_held();
    draw_beveled_box(elements, jx, jy, jw, jh, jw / 2.0, jump_active, gs);
    draw_pixel_jump(elements, jx + jw / 2.0, jy + jh / 2.0, 2.0 * gs, WHITE);

    let (ax, ay, aw, ah) = layout.button_rect(TouchButton::Primary);
    let pri_active = touch.is_primary_held();
    draw_beveled_box(elements, ax, ay, aw, ah, 8.0 * gs, pri_active, gs);
    draw_button_text(
        elements,
        ax + aw / 2.0,
        ay + ah / 2.0,
        "ATK",
        FONT_SIZE * gs * 0.90,
    );

    let (ux, uy, uw, uh) = layout.button_rect(TouchButton::Secondary);
    let sec_active = touch.is_secondary_held();
    draw_beveled_box(elements, ux, uy, uw, uh, 8.0 * gs, sec_active, gs);
    draw_button_text(
        elements,
        ux + uw / 2.0,
        uy + uh / 2.0,
        "USE",
        FONT_SIZE * gs * 0.90,
    );

    let (sx, sy, sw, sh) = layout.button_rect(TouchButton::Sneak);
    let snk_active = touch.is_sneak_locked();
    draw_beveled_box(elements, sx, sy, sw, sh, 6.0 * gs, snk_active, gs);
    draw_button_text(
        elements,
        sx + sw / 2.0,
        sy + sh / 2.0,
        "SNK",
        FONT_SIZE * gs * 0.80,
    );

    let (px, py, pw, ph) = layout.button_rect(TouchButton::Pause);
    draw_beveled_box(elements, px, py, pw, ph, 4.0 * gs, false, gs);
    draw_pixel_pause(elements, px + pw / 2.0, py + ph / 2.0, 1.8 * gs, WHITE);

    let (cx, cy, cw, ch) = layout.button_rect(TouchButton::Chat);
    draw_beveled_box(elements, cx, cy, cw, ch, 4.0 * gs, false, gs);
    draw_button_text(
        elements,
        cx + cw / 2.0,
        cy + ch / 2.0,
        "CHAT",
        FONT_SIZE * gs * 0.70,
    );

    for &btn in &[
        TouchButton::Debug,
        TouchButton::ThirdPerson,
        TouchButton::PlayerList,
        TouchButton::Keyboard,
        TouchButton::MouseToggle,
    ] {
        let (bx, by, bw, bh) = layout.button_rect(btn);
        let active = match btn {
            TouchButton::PlayerList => touch.player_list_held(),
            TouchButton::MouseToggle => touch.is_virtual_mouse_mode(),
            _ => false,
        };
        draw_beveled_box(elements, bx, by, bw, bh, 4.0 * gs, active, gs);
        draw_button_text(
            elements,
            bx + bw / 2.0,
            by + bh / 2.0,
            btn.label(),
            FONT_SIZE * gs * 0.72,
        );
    }

    if touch.is_virtual_mouse_mode() {
        push_virtual_cursor(elements, touch.virtual_cursor_position(), gs);
    }
}

fn draw_beveled_box(
    elements: &mut Vec<MenuElement>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    active: bool,
    gs: f32,
) {
    let (bg, border) = if active {
        (BG_ACTIVE, BORDER_ACTIVE)
    } else {
        (BG_IDLE, BORDER_IDLE)
    };
    elements.push(MenuElement::Rect {
        x,
        y,
        w,
        h,
        corner_radius: radius,
        color: border,
    });
    let bw = 1.0 * gs;
    elements.push(MenuElement::Rect {
        x: x + bw,
        y: y + bw,
        w: w - 2.0 * bw,
        h: h - 2.0 * bw,
        corner_radius: (radius - bw).max(0.0),
        color: bg,
    });
}

fn draw_button_text(elements: &mut Vec<MenuElement>, cx: f32, cy: f32, text: &str, scale: f32) {
    elements.push(MenuElement::Text {
        x: cx,
        y: cy - scale / 2.0,
        text: text.to_owned(),
        scale,
        color: WHITE,
        centered: true,
    });
}

fn draw_pixel_arrow_up(elements: &mut Vec<MenuElement>, cx: f32, cy: f32, u: f32, color: [f32; 4]) {
    elements.push(MenuElement::Rect {
        x: cx - u,
        y: cy - 3.5 * u,
        w: 2.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 2.0 * u,
        y: cy - 2.5 * u,
        w: 4.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 3.5 * u,
        y: cy - 1.5 * u,
        w: 7.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - u,
        y: cy - 0.5 * u,
        w: 2.0 * u,
        h: 3.5 * u,
        corner_radius: 0.0,
        color,
    });
}

fn draw_pixel_arrow_down(
    elements: &mut Vec<MenuElement>,
    cx: f32,
    cy: f32,
    u: f32,
    color: [f32; 4],
) {
    elements.push(MenuElement::Rect {
        x: cx - u,
        y: cy - 3.0 * u,
        w: 2.0 * u,
        h: 3.5 * u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 3.5 * u,
        y: cy + 0.5 * u,
        w: 7.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 2.0 * u,
        y: cy + 1.5 * u,
        w: 4.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - u,
        y: cy + 2.5 * u,
        w: 2.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
}

fn draw_pixel_arrow_left(
    elements: &mut Vec<MenuElement>,
    cx: f32,
    cy: f32,
    u: f32,
    color: [f32; 4],
) {
    elements.push(MenuElement::Rect {
        x: cy_trans(cx - 3.5 * u),
        y: cy - u,
        w: u,
        h: 2.0 * u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 2.5 * u,
        y: cy - 2.0 * u,
        w: u,
        h: 4.0 * u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 1.5 * u,
        y: cy - 3.5 * u,
        w: u,
        h: 7.0 * u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 0.5 * u,
        y: cy - u,
        w: 3.5 * u,
        h: 2.0 * u,
        corner_radius: 0.0,
        color,
    });
}

fn cy_trans(v: f32) -> f32 {
    v
}

fn draw_pixel_arrow_right(
    elements: &mut Vec<MenuElement>,
    cx: f32,
    cy: f32,
    u: f32,
    color: [f32; 4],
) {
    elements.push(MenuElement::Rect {
        x: cx - 3.0 * u,
        y: cy - u,
        w: 3.5 * u,
        h: 2.0 * u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx + 0.5 * u,
        y: cy - 3.5 * u,
        w: u,
        h: 7.0 * u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx + 1.5 * u,
        y: cy - 2.0 * u,
        w: u,
        h: 4.0 * u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx + 2.5 * u,
        y: cy - u,
        w: u,
        h: 2.0 * u,
        corner_radius: 0.0,
        color,
    });
}

fn draw_pixel_diamond(elements: &mut Vec<MenuElement>, cx: f32, cy: f32, u: f32, color: [f32; 4]) {
    elements.push(MenuElement::Rect {
        x: cx - u,
        y: cy - 3.0 * u,
        w: 2.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 2.5 * u,
        y: cy - 2.0 * u,
        w: 5.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 3.5 * u,
        y: cy - u,
        w: 7.0 * u,
        h: 2.0 * u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 2.5 * u,
        y: cy + u,
        w: 5.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - u,
        y: cy + 2.0 * u,
        w: 2.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
}

fn draw_pixel_jump(elements: &mut Vec<MenuElement>, cx: f32, cy: f32, u: f32, color: [f32; 4]) {
    elements.push(MenuElement::Rect {
        x: cx - u,
        y: cy - 4.0 * u,
        w: 2.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 2.5 * u,
        y: cy - 3.0 * u,
        w: 5.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 4.0 * u,
        y: cy - 2.0 * u,
        w: 8.0 * u,
        h: u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - u,
        y: cy - u,
        w: 2.0 * u,
        h: 2.5 * u,
        corner_radius: 0.0,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx - 4.0 * u,
        y: cy + 2.5 * u,
        w: 8.0 * u,
        h: 1.2 * u,
        corner_radius: 0.0,
        color,
    });
}

fn draw_pixel_dots(elements: &mut Vec<MenuElement>, cx: f32, cy: f32, u: f32, color: [f32; 4]) {
    let size = 2.0 * u;
    for &dx in &[-3.5 * u, 0.0, 3.5 * u] {
        elements.push(MenuElement::Rect {
            x: cx + dx - size / 2.0,
            y: cy - size / 2.0,
            w: size,
            h: size,
            corner_radius: 0.0,
            color,
        });
    }
}

fn draw_pixel_pause(elements: &mut Vec<MenuElement>, cx: f32, cy: f32, u: f32, color: [f32; 4]) {
    let w = 1.8 * u;
    let h = 6.0 * u;
    elements.push(MenuElement::Rect {
        x: cx - 2.2 * u,
        y: cy - h / 2.0,
        w,
        h,
        corner_radius: 0.5 * u,
        color,
    });
    elements.push(MenuElement::Rect {
        x: cx + 0.4 * u,
        y: cy - h / 2.0,
        w,
        h,
        corner_radius: 0.5 * u,
        color,
    });
}

fn push_virtual_cursor(elements: &mut Vec<MenuElement>, pos: Vec2, gs: f32) {
    let size = 12.0 * gs;
    elements.push(MenuElement::Rect {
        x: pos.x - 1.0 * gs,
        y: pos.y - 1.0 * gs,
        w: size + 2.0 * gs,
        h: size + 2.0 * gs,
        corner_radius: 1.0 * gs,
        color: CURSOR_OUTLINE,
    });
    elements.push(MenuElement::Rect {
        x: pos.x,
        y: pos.y,
        w: size,
        h: size,
        corner_radius: 0.5 * gs,
        color: CURSOR_BODY,
    });
}
