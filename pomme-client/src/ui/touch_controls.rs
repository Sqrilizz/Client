use glam::Vec2;

use crate::app::input::touch::{TouchButton, TouchControls, TouchLayout};
use crate::renderer::pipelines::menu_overlay::MenuElement;
use crate::ui::common::{FONT_SIZE, WHITE};

const BASE: [f32; 4] = [0.0, 0.0, 0.0, 0.40];
const BASE_ACTIVE: [f32; 4] = [0.2, 0.4, 0.7, 0.65];
const CURSOR_BODY: [f32; 4] = [1.0, 1.0, 1.0, 0.95];
const CURSOR_OUTLINE: [f32; 4] = [0.0, 0.0, 0.0, 0.95];

pub fn build_touch_controls(
    elements: &mut Vec<MenuElement>,
    layout: &TouchLayout,
    touch: &TouchControls,
    menu_mode: bool,
) {
    let gs = layout.gs;
    if !menu_mode {
        let dpad = layout.dpad_home();
        let step = layout.dpad_step();
        let radius = layout.button_radius(TouchButton::Forward);
        let arm_len = step + radius;
        let bar_width = 2.0 * radius;
        elements.push(MenuElement::Rect {
            x: dpad.x - bar_width / 2.0,
            y: dpad.y - arm_len,
            w: bar_width,
            h: 2.0 * arm_len,
            corner_radius: radius,
            color: [0.0, 0.0, 0.0, 0.25],
        });
        elements.push(MenuElement::Rect {
            x: dpad.x - arm_len,
            y: dpad.y - bar_width / 2.0,
            w: 2.0 * arm_len,
            h: bar_width,
            corner_radius: radius,
            color: [0.0, 0.0, 0.0, 0.25],
        });
    }

    let buttons: &[TouchButton] = if menu_mode {
        if touch.is_virtual_mouse_mode() {
            &[
                TouchButton::Keyboard,
                TouchButton::MouseToggle,
                TouchButton::Primary,
                TouchButton::Secondary,
            ]
        } else {
            &[TouchButton::Keyboard, TouchButton::MouseToggle]
        }
    } else {
        &TouchButton::ALL
    };

    for &button in buttons {
        let centre = layout.button_centre(button);
        let radius = layout.button_radius(button);
        let active = match button {
            TouchButton::Sneak | TouchButton::DpadCenter => touch.is_sneak_locked(),
            TouchButton::MouseToggle => touch.is_virtual_mouse_mode(),
            TouchButton::Forward => touch.is_forward_held(),
            TouchButton::Backward => touch.is_backward_held(),
            TouchButton::Left => touch.is_left_held(),
            TouchButton::Right => touch.is_right_held(),
            TouchButton::Jump => touch.jump_held(),
            TouchButton::Primary => touch.is_primary_held(),
            TouchButton::Secondary => touch.is_secondary_held(),
            _ => false,
        };
        let color = if active { BASE_ACTIVE } else { BASE };
        push_circle(elements, centre, radius, color);
        let font_scale = match button {
            TouchButton::Forward
            | TouchButton::Backward
            | TouchButton::Left
            | TouchButton::Right => FONT_SIZE * gs * 1.6,
            TouchButton::DpadCenter => FONT_SIZE * gs * 1.3,
            TouchButton::Jump => FONT_SIZE * gs * 1.1,
            _ => FONT_SIZE * gs * 0.95,
        };
        elements.push(MenuElement::Text {
            x: centre.x,
            y: centre.y - font_scale / 2.0,
            text: button.label().to_owned(),
            scale: font_scale,
            color: WHITE,
            centered: true,
        });
    }

    if touch.is_virtual_mouse_mode() {
        let pos = touch.virtual_cursor_position();
        push_virtual_cursor(elements, pos, gs);
    }
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

fn push_circle(elements: &mut Vec<MenuElement>, centre: Vec2, radius: f32, color: [f32; 4]) {
    elements.push(MenuElement::Rect {
        x: centre.x - radius,
        y: centre.y - radius,
        w: 2.0 * radius,
        h: 2.0 * radius,
        corner_radius: radius,
        color,
    });
}
