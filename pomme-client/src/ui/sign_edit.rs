use crate::position::BlockPos;
use crate::renderer::pipelines::menu_overlay::MenuElement;
use crate::ui::common::{FONT_SIZE, WHITE, push_button, push_field_text, push_menu_background};
use crate::ui::text::TextSpan;
use crate::ui::text_edit::{SystemClipboard, TextFieldState, TextInputEvent};

const MAX_SIGN_LINE_LENGTH: usize = 384;
/// The sign panel's width in GUI units.
const SIGN_W: f32 = 192.0;
/// Horizontal padding inside the panel on each side.
const SIGN_PAD: f32 = 8.0;

/// The inner text width the line fields measure against, at this scale.
pub fn line_inner_w(gs: f32) -> f32 {
    (SIGN_W - 2.0 * SIGN_PAD) * gs
}

pub struct SignEditState {
    pub pos: BlockPos,
    pub is_front_text: bool,
    pub lines: [TextFieldState; 4],
    pub current_line: usize,
    inner_w: f32,
    pub wood_color: [f32; 4],
    pub text_color: [f32; 4],
}

/// How the editor closed: `Submit` sends the current text as
/// `sign_update`, `Cancel` just closes (server text stays).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SignEditResult {
    Submit,
    Cancel,
}

impl SignEditState {
    pub fn new(
        pos: BlockPos,
        is_front_text: bool,
        initial_lines: [String; 4],
        wood_color: [f32; 4],
        text_color: [f32; 4],
        gs: f32,
    ) -> Self {
        let inner_w = line_inner_w(gs);
        let mut lines = [
            TextFieldState::new(MAX_SIGN_LINE_LENGTH),
            TextFieldState::new(MAX_SIGN_LINE_LENGTH),
            TextFieldState::new(MAX_SIGN_LINE_LENGTH),
            TextFieldState::new(MAX_SIGN_LINE_LENGTH),
        ];
        for (i, text) in initial_lines.into_iter().enumerate() {
            // A zero width fn keeps `display_pos` at the line start, so the
            // panel opens with the text shown from its left edge (the render
            // pass measures with the real width).
            lines[i].set_value(&text, inner_w, &|_| 0.0);
        }
        lines[0].set_focused(true);
        Self {
            pos,
            is_front_text,
            lines,
            current_line: 0,
            inner_w,
            wood_color,
            text_color,
        }
    }

    pub fn set_current_line(&mut self, next: usize) {
        self.lines[self.current_line].set_focused(false);
        self.current_line = next % 4;
        self.lines[self.current_line].set_focused(true);
    }

    pub fn handle_text_input(
        &mut self,
        events: &[TextInputEvent],
        width_fn: &dyn Fn(&str) -> f32,
    ) -> bool {
        let inner_w = self.inner_w;
        let mut clipboard = SystemClipboard;
        let mut finish = false;
        for ev in events {
            match ev {
                TextInputEvent::Key { code, mods } => match code {
                    winit::keyboard::KeyCode::ArrowUp => {
                        self.set_current_line((self.current_line + 3) % 4);
                    }
                    winit::keyboard::KeyCode::ArrowDown => {
                        self.set_current_line((self.current_line + 1) % 4);
                    }
                    winit::keyboard::KeyCode::Tab => {
                        if mods.shift {
                            self.set_current_line((self.current_line + 3) % 4);
                        } else {
                            self.set_current_line((self.current_line + 1) % 4);
                        }
                    }
                    winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                        if self.current_line < 3 {
                            self.set_current_line(self.current_line + 1);
                        } else {
                            finish = true;
                        }
                    }
                    winit::keyboard::KeyCode::Backspace
                        if self.lines[self.current_line].value().is_empty()
                            && self.current_line > 0 =>
                    {
                        self.set_current_line(self.current_line - 1);
                    }
                    _ => {
                        self.lines[self.current_line].key_pressed(
                            *code,
                            mods,
                            &mut clipboard,
                            inner_w,
                            width_fn,
                        );
                    }
                },
                TextInputEvent::Char(c) => {
                    self.lines[self.current_line].char_typed(*c, inner_w, width_fn);
                }
            }
        }
        finish
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &mut self,
        elements: &mut Vec<MenuElement>,
        screen_w: f32,
        screen_h: f32,
        gs: f32,
        cursor: (f32, f32),
        clicked: bool,
        width_fn: &dyn Fn(&str) -> f32,
    ) -> Option<SignEditResult> {
        push_menu_background(elements, screen_w, screen_h);

        let fs = FONT_SIZE * gs;
        let title_y = ((screen_h / 4.0) - 20.0 * gs).max(10.0 * gs);
        elements.push(MenuElement::Text {
            x: screen_w / 2.0,
            y: title_y,
            text: "Edit Sign Message".into(),
            scale: fs,
            color: WHITE,
            centered: true,
        });

        let sign_w = 192.0 * gs;
        let sign_h = 96.0 * gs;
        let sign_x = (screen_w - sign_w) / 2.0;
        let sign_y = (screen_h - sign_h) / 2.0 - 15.0 * gs;

        let outline_thick = 2.0 * gs;
        let border_color = [
            self.wood_color[0] * 0.45,
            self.wood_color[1] * 0.45,
            self.wood_color[2] * 0.45,
            1.0,
        ];

        let post_w = 14.0 * gs;
        let post_h = 24.0 * gs;
        let post_x = (screen_w - post_w) / 2.0;
        let post_y = sign_y + sign_h + outline_thick;
        elements.push(MenuElement::Rect {
            x: post_x,
            y: post_y,
            w: post_w,
            h: post_h,
            corner_radius: 0.0,
            color: border_color,
        });

        elements.push(MenuElement::Rect {
            x: sign_x - outline_thick,
            y: sign_y - outline_thick,
            w: sign_w + 2.0 * outline_thick,
            h: sign_h + 2.0 * outline_thick,
            corner_radius: 0.0,
            color: border_color,
        });

        let plank_h = sign_h / 4.0;
        for p in 0..4 {
            let py = sign_y + (p as f32) * plank_h;
            let shade = if p % 2 == 0 { 1.0 } else { 0.93 };
            let plank_color = [
                self.wood_color[0] * shade,
                self.wood_color[1] * shade,
                self.wood_color[2] * shade,
                1.0,
            ];
            elements.push(MenuElement::Rect {
                x: sign_x,
                y: py,
                w: sign_w,
                h: plank_h,
                corner_radius: 0.0,
                color: plank_color,
            });
            if p > 0 {
                elements.push(MenuElement::Rect {
                    x: sign_x,
                    y: py,
                    w: sign_w,
                    h: 1.0 * gs,
                    corner_radius: 0.0,
                    color: [0.0, 0.0, 0.0, 0.18],
                });
            }
        }

        let line_h = 18.0 * gs;
        let lines_start_y = sign_y + (sign_h - 4.0 * line_h) / 2.0;
        let inner_w = sign_w - 16.0 * gs;

        for i in 0..4 {
            let line_y = lines_start_y + (i as f32) * line_h;
            let line_hit_x = sign_x + 4.0 * gs;
            let line_hit_w = sign_w - 8.0 * gs;

            let text_w = width_fn(self.lines[i].value());
            let text_x = sign_x + ((sign_w - text_w) / 2.0).max(8.0 * gs);
            let text_y = line_y + (line_h - fs) / 2.0;

            if clicked
                && cursor.0 >= line_hit_x
                && cursor.0 <= line_hit_x + line_hit_w
                && cursor.1 >= line_y
                && cursor.1 < line_y + line_h
            {
                self.set_current_line(i);
                let rel_x = cursor.0 - text_x;
                let click_pos = self.lines[i].pos_from_click(rel_x, inner_w, width_fn);
                self.lines[i].move_cursor_to(click_pos, false, inner_w, width_fn);
            }

            let val = self.lines[i].value().to_string();
            if i == self.current_line {
                let info = self.lines[i].render_info(inner_w, true, width_fn);
                let shown = &self.lines[i].value()[info.display_start..info.display_end];
                let spans = [TextSpan::new(shown.to_string(), self.text_color)];

                elements.push(MenuElement::ScissorPush {
                    x: line_hit_x,
                    y: line_y,
                    w: line_hit_w,
                    h: line_h,
                });
                push_field_text(
                    elements,
                    &info,
                    shown,
                    Some(&spans),
                    text_x,
                    text_y,
                    fs,
                    gs,
                    gs,
                    self.text_color,
                    None,
                    width_fn,
                );
                elements.push(MenuElement::ScissorPop);
            } else if !val.is_empty() {
                elements.push(MenuElement::Text {
                    x: text_x,
                    y: text_y,
                    text: val,
                    scale: fs,
                    color: self.text_color,
                    centered: false,
                });
            }
        }

        // Vanilla 1.20.5+: Done and Cancel side by side (98x20 each).
        let btn_w = 98.0 * gs;
        let btn_h = 20.0 * gs;
        let btn_gap = 4.0 * gs;
        let pair_w = 2.0 * btn_w + btn_gap;
        let btn_x = (screen_w - pair_w) / 2.0;
        let btn_y = sign_y + sign_h + 30.0 * gs;
        let done_hovered = push_button(
            elements, cursor, btn_x, btn_y, btn_w, btn_h, gs, fs, "Done", true,
        );
        let cancel_hovered = push_button(
            elements,
            cursor,
            btn_x + btn_w + btn_gap,
            btn_y,
            btn_w,
            btn_h,
            gs,
            fs,
            "Cancel",
            true,
        );
        if clicked && done_hovered {
            return Some(SignEditResult::Submit);
        }
        if clicked && cancel_hovered {
            return Some(SignEditResult::Cancel);
        }
        None
    }

    pub fn lines_as_strings(&self) -> [String; 4] {
        [
            self.lines[0].value().to_string(),
            self.lines[1].value().to_string(),
            self.lines[2].value().to_string(),
            self.lines[3].value().to_string(),
        ]
    }
}
