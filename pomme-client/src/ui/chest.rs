//! The single-grid container screens: the generic 9xN chest (vanilla
//! `ContainerScreen` + `ChestMenu`), the shulker box (`ShulkerBoxScreen`),
//! the hopper (`HopperScreen`) and the dispenser and dropper
//! (`DispenserScreen`): contents slots first, then player inventory and
//! hotbar.

use std::time::Instant;

use pomme_item::ItemStack;

use super::common::SLOT_STRIDE;
use super::container::{
    ContainerInput, ContainerResult, DragState, Panel, SlotCtx, centred_title_x, gesture_result,
    push_backdrop, push_clipped_sprite, push_cursor_stack, push_panel,
};
use crate::player::menu_click::ContainerKind;
use crate::renderer::pipelines::menu_overlay::{MenuElement, SpriteId};

/// The generic_54 texture's top (rows) piece is 176x125; the player-inventory
/// piece below it is 176x96 and completes any row count's background.
const TOP_TEX_H: f32 = 125.0;
const BOTTOM_TEX_H: f32 = 96.0;

#[allow(clippy::too_many_arguments)]
pub fn build_chest(
    elements: &mut Vec<MenuElement>,
    screen_w: f32,
    screen_h: f32,
    cursor: (f32, f32),
    input: &ContainerInput,
    rows: u8,
    slots: &[ItemStack],
    title: &str,
    title_spans: &[crate::ui::text::TextSpan],
    cursor_item: &ItemStack,
    drag: &mut Option<DragState>,
    last_click: &mut Option<(u16, Instant)>,
    gs: f32,
) -> ContainerResult {
    let rows_h = rows as f32 * SLOT_STRIDE;
    let panel = push_backdrop(elements, screen_w, screen_h, gs, 176.0, 114.0 + rows_h);
    // Vanilla ContainerScreen draws generic_54 in two pieces: the top slice
    // cut off after the last contents row, then the player-inventory strip.
    let split = rows_h + 17.0;
    push_clipped_sprite(
        elements,
        &panel,
        SpriteId::Generic54Top,
        [0.0, 0.0, 176.0, TOP_TEX_H],
        [0.0, 0.0, 176.0, split],
    );
    panel.image(
        elements,
        SpriteId::Generic54Bottom,
        0.0,
        split,
        176.0,
        BOTTOM_TEX_H,
    );
    if title_spans.is_empty() {
        panel.label(elements, 8.0, 6.0, title);
    } else {
        panel.label_spans(elements, 8.0, 6.0, title_spans);
    }
    panel.inventory_label(elements, rows_h + 20.0);

    build_contents(
        elements,
        &panel,
        cursor,
        input,
        ContainerKind::Chest { rows },
        CHEST_GRID,
        rows_h + 31.0,
        slots,
        cursor_item,
        drag,
        last_click,
    )
}

/// A contents grid: its top-left slot in GUI units and its column count.
struct Grid {
    x: f32,
    y: f32,
    cols: u16,
}

const CHEST_GRID: Grid = Grid {
    x: 8.0,
    y: 18.0,
    cols: 9,
};

/// The shulker box, hopper and dispenser screens, each one background sprite
/// over a contents grid. `kind` is one of those three menus.
#[allow(clippy::too_many_arguments)]
pub fn build_grid_screen(
    elements: &mut Vec<MenuElement>,
    screen_w: f32,
    screen_h: f32,
    cursor: (f32, f32),
    input: &ContainerInput,
    kind: ContainerKind,
    slots: &[ItemStack],
    title: &str,
    title_spans: &[crate::ui::text::TextSpan],
    cursor_item: &ItemStack,
    drag: &mut Option<DragState>,
    last_click: &mut Option<(u16, Instant)>,
    gs: f32,
    text_width_fn: &dyn Fn(&str, f32) -> f32,
) -> ContainerResult {
    let (height, sprite, grid, main_y) = match kind {
        ContainerKind::ShulkerBox => (167.0, SpriteId::ShulkerBoxBackground, CHEST_GRID, 84.0),
        ContainerKind::Hopper => (
            133.0,
            SpriteId::HopperBackground,
            Grid {
                x: 44.0,
                y: 20.0,
                cols: 5,
            },
            51.0,
        ),
        _ => (
            166.0,
            SpriteId::DispenserBackground,
            Grid {
                x: 62.0,
                y: 17.0,
                cols: 3,
            },
            84.0,
        ),
    };
    let panel = push_panel(elements, screen_w, screen_h, gs, height, sprite);
    // `DispenserScreen` centres its title; the others keep the default.
    let title_x = if matches!(kind, ContainerKind::Dispenser) {
        centred_title_x(title, text_width_fn)
    } else {
        8.0
    };
    if title_spans.is_empty() {
        panel.label(elements, title_x, 6.0, title);
    } else {
        panel.label_spans(elements, title_x, 6.0, title_spans);
    }
    panel.inventory_label(elements, height - 94.0);

    build_contents(
        elements,
        &panel,
        cursor,
        input,
        kind,
        grid,
        main_y,
        slots,
        cursor_item,
        drag,
        last_click,
    )
}

/// The contents `grid`, the player rows at `main_y`, and the shared gesture
/// tail.
#[allow(clippy::too_many_arguments)]
fn build_contents(
    elements: &mut Vec<MenuElement>,
    panel: &Panel,
    cursor: (f32, f32),
    input: &ContainerInput,
    kind: ContainerKind,
    grid: Grid,
    main_y: f32,
    slots: &[ItemStack],
    cursor_item: &ItemStack,
    drag: &mut Option<DragState>,
    last_click: &mut Option<(u16, Instant)>,
) -> ContainerResult {
    let mut ctx = SlotCtx::new(elements, panel, cursor, kind, slots, cursor_item, drag);

    let main_base = kind.inv_start() as u16;
    ctx.player_rows(slots, main_base, main_base + 27, main_y);

    for num in 0..main_base {
        let item = slots.get(num as usize).unwrap_or(&ItemStack::Empty);
        ctx.slot(
            grid.x + (num % grid.cols) as f32 * SLOT_STRIDE,
            grid.y + (num / grid.cols) as f32 * SLOT_STRIDE,
            item,
            None,
            num,
        );
    }

    let (hovered, shown_cursor) = ctx.finish(cursor_item);
    push_cursor_stack(elements, cursor, panel.scale, &shown_cursor);

    gesture_result(
        input,
        hovered,
        panel,
        cursor,
        kind,
        slots,
        cursor_item,
        drag,
        last_click,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::text::TextSpan;

    #[test]
    fn chest_emits_flat_spans_when_title_spans_present() {
        let mut elements = Vec::new();
        let input = ContainerInput {
            left_pressed: false,
            right_pressed: false,
            middle_pressed: false,
            left_held: false,
            right_held: false,
            shift: false,
            hotbar_swap: None,
            swap_offhand: false,
            throw: false,
            throw_all: false,
        };
        let mut drag = None;
        let mut last_click = None;
        let spans = vec![TextSpan {
            text: "\u{e682}".into(),
            color: [1.0, 1.0, 1.0, 1.0],
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            obfuscated: false,
            shadow_color: None,
            font: Some("hopneo:gui".into()),
            inline_object: None,
            component_style: None,
        }];
        build_chest(
            &mut elements,
            800.0,
            600.0,
            (0.0, 0.0),
            &input,
            3,
            &[],
            "Shop",
            &spans,
            &ItemStack::Empty,
            &mut drag,
            &mut last_click,
            1.0,
        );
        let has_flat_spans = elements.iter().any(|elem| matches!(elem, MenuElement::TextSpansFlat { spans: s, .. } if s == &spans));
        assert!(has_flat_spans);
    }

    #[test]
    fn chest_emits_flat_text_when_title_spans_empty() {
        let mut elements = Vec::new();
        let input = ContainerInput {
            left_pressed: false,
            right_pressed: false,
            middle_pressed: false,
            left_held: false,
            right_held: false,
            shift: false,
            hotbar_swap: None,
            swap_offhand: false,
            throw: false,
            throw_all: false,
        };
        let mut drag = None;
        let mut last_click = None;
        build_chest(
            &mut elements,
            800.0,
            600.0,
            (0.0, 0.0),
            &input,
            3,
            &[],
            "Chest",
            &[],
            &ItemStack::Empty,
            &mut drag,
            &mut last_click,
            1.0,
        );
        let has_flat_text = elements.iter().any(|elem| matches!(elem, MenuElement::TextFlat { text, .. } if text == "Chest"));
        assert!(has_flat_text);
    }
}
