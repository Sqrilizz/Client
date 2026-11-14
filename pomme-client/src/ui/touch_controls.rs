use glam::Vec2;

use crate::app::input::touch::{
    BUTTON_RADIUS, STICK_RADIUS, TouchButton, TouchControls, TouchLayout,
};
use crate::renderer::pipelines::menu_overlay::MenuElement;
use crate::ui::common::{FONT_SIZE, WHITE};

const BASE: [f32; 4] = [0.0, 0.0, 0.0, 0.40];
const BASE_ACTIVE: [f32; 4] = [0.2, 0.4, 0.7, 0.65];
const KNOB: [f32; 4] = [1.0, 1.0, 1.0, 0.50];
const CURSOR_BODY: [f32; 4] = [1.0, 1.0, 1.0, 0.95];
const CURSOR_OUTLINE: [f32; 4] = [0.0, 0.0, 0.0, 0.95];

pub fn build_touch_controls(
    elements: &mut Vec<MenuElement>,
    layout: &TouchLayout,
    touch: &TouchControls,
) {
    let gs = layout.gs;
    let radius = STICK_RADIUS * gs;
    let base = touch.stick_origin().unwrap_or_else(|| layout.stick_home());
    push_circle(elements, base, radius, BASE);
    let deflection = touch.stick().map_or(Vec2::ZERO, |v| Vec2::new(v.x, -v.y));
    push_circle(elements, base + deflection * radius, radius * 0.45, KNOB);

    for button in TouchButton::ALL {
        let centre = layout.button_centre(button);
        let color = if (button == TouchButton::Sneak && touch.is_sneak_locked())
            || (button == TouchButton::MouseToggle && touch.is_virtual_mouse_mode())
        {
            BASE_ACTIVE
        } else {
            BASE
        };
        push_circle(elements, centre, BUTTON_RADIUS * gs, color);
        elements.push(MenuElement::Text {
            x: centre.x,
            y: centre.y - FONT_SIZE * gs / 2.0,
            text: button.label().to_owned(),
            scale: FONT_SIZE * gs,
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
