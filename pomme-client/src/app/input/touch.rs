use std::collections::HashMap;
use std::time::{Duration, Instant};

use glam::Vec2;
use winit::event::TouchPhase;

use super::action::Action;

const TAP_TIME: Duration = Duration::from_millis(250);
const HOLD_TIME: Duration = Duration::from_millis(300);
const STILL_SLOP: f32 = 4.0;
pub const STICK_RADIUS: f32 = 30.0;
const LOOK_SCALE: f32 = 2.5;
const STICK_ZONE: f32 = 0.4;
pub const BUTTON_RADIUS: f32 = 20.0;

#[derive(Clone, Copy)]
pub struct TouchLayout {
    pub width: f32,
    pub height: f32,
    pub gs: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchButton {
    Forward,
    Backward,
    Left,
    Right,
    DpadCenter,
    Jump,
    Chat,
    Inventory,
    Pause,
    Keyboard,
    Debug,
    ThirdPerson,
    PlayerList,
    Sneak,
    Primary,
    Secondary,
    MouseToggle,
}

impl TouchButton {
    pub const ALL: [Self; 17] = [
        Self::Forward,
        Self::Backward,
        Self::Left,
        Self::Right,
        Self::DpadCenter,
        Self::Jump,
        Self::Chat,
        Self::Inventory,
        Self::Pause,
        Self::Keyboard,
        Self::Debug,
        Self::ThirdPerson,
        Self::PlayerList,
        Self::Sneak,
        Self::Primary,
        Self::Secondary,
        Self::MouseToggle,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Forward => "^",
            Self::Backward => "v",
            Self::Left => "<",
            Self::Right => ">",
            Self::DpadCenter => "o",
            Self::Jump => "JUMP",
            Self::Chat => "CHAT",
            Self::Inventory => "INV",
            Self::Pause => "| |",
            Self::Keyboard => "KBD",
            Self::Debug => "F3",
            Self::ThirdPerson => "F5",
            Self::PlayerList => "TAB",
            Self::Sneak => "SNK",
            Self::Primary => "PRI",
            Self::Secondary => "SEC",
            Self::MouseToggle => "MOUSE",
        }
    }

    fn tapped_action(self) -> Option<Action> {
        match self {
            Self::Forward => None,
            Self::Backward => None,
            Self::Left => None,
            Self::Right => None,
            Self::DpadCenter => None,
            Self::Jump => None,
            Self::Chat => Some(Action::OpenChat),
            Self::Inventory => Some(Action::ToggleInventory),
            Self::Pause => Some(Action::OpenMenu),
            Self::Keyboard => None,
            Self::Debug => Some(Action::ToggleOverlay),
            Self::ThirdPerson => Some(Action::ChangePerspective),
            Self::PlayerList => None,
            Self::Sneak => None,
            Self::Primary => None,
            Self::Secondary => None,
            Self::MouseToggle => None,
        }
    }
}

impl TouchLayout {
    pub fn dpad_cell(&self) -> f32 {
        38.0 * self.gs
    }

    pub fn dpad_home(&self) -> Vec2 {
        let cell = self.dpad_cell();
        let margin = 20.0 * self.gs;
        Vec2::new(margin + 1.5 * cell, self.height - margin - 1.5 * cell)
    }

    pub fn dpad_step(&self) -> f32 {
        self.dpad_cell()
    }

    pub fn button_rect(&self, button: TouchButton) -> (f32, f32, f32, f32) {
        let gs = self.gs;
        let dpad = self.dpad_home();
        let cell = self.dpad_cell();
        let top_y = 6.0 * gs;
        let top_h = 20.0 * gs;
        let cx = self.width / 2.0;
        let hotbar_w = 182.0 * gs;
        let hotbar_x = (self.width - hotbar_w) / 2.0;
        let hotbar_y = self.height - 22.0 * gs;
        let rm = 24.0 * gs;
        let bm = 24.0 * gs;
        let jump_size = 52.0 * gs;
        let btn_w = 42.0 * gs;
        let btn_h = 42.0 * gs;
        let snk_size = 36.0 * gs;

        match button {
            TouchButton::Forward => (dpad.x - cell / 2.0, dpad.y - 1.5 * cell, cell, cell),
            TouchButton::Backward => (dpad.x - cell / 2.0, dpad.y + 0.5 * cell, cell, cell),
            TouchButton::Left => (dpad.x - 1.5 * cell, dpad.y - cell / 2.0, cell, cell),
            TouchButton::Right => (dpad.x + 0.5 * cell, dpad.y - cell / 2.0, cell, cell),
            TouchButton::DpadCenter => (dpad.x - cell / 2.0, dpad.y - cell / 2.0, cell, cell),
            TouchButton::Jump => (
                self.width - rm - jump_size,
                self.height - bm - jump_size,
                jump_size,
                jump_size,
            ),
            TouchButton::Primary => (
                self.width - rm - jump_size - btn_w - 12.0 * gs,
                self.height - bm - btn_h,
                btn_w,
                btn_h,
            ),
            TouchButton::Secondary => (
                self.width - rm - btn_w,
                self.height - bm - jump_size - btn_h - 12.0 * gs,
                btn_w,
                btn_h,
            ),
            TouchButton::Sneak => (
                self.width - rm - jump_size - btn_w - 12.0 * gs + (btn_w - snk_size) / 2.0,
                self.height - bm - jump_size - btn_h - 12.0 * gs + (btn_h - snk_size) / 2.0,
                snk_size,
                snk_size,
            ),
            TouchButton::Inventory => (
                hotbar_x + hotbar_w + 4.0 * gs,
                hotbar_y,
                22.0 * gs,
                22.0 * gs,
            ),
            TouchButton::Pause => (cx + 28.0 * gs, top_y, 26.0 * gs, top_h),
            TouchButton::Chat => (cx - 28.0 * gs - 28.0 * gs, top_y, 28.0 * gs, top_h),
            TouchButton::Debug => (8.0 * gs, top_y, 28.0 * gs, top_h),
            TouchButton::ThirdPerson => (38.0 * gs, top_y, 28.0 * gs, top_h),
            TouchButton::PlayerList => (68.0 * gs, top_y, 32.0 * gs, top_h),
            TouchButton::Keyboard => (102.0 * gs, top_y, 34.0 * gs, top_h),
            TouchButton::MouseToggle => (138.0 * gs, top_y, 44.0 * gs, top_h),
        }
    }

    pub fn button_centre(&self, button: TouchButton) -> Vec2 {
        let (x, y, w, h) = self.button_rect(button);
        Vec2::new(x + w / 2.0, y + h / 2.0)
    }

    pub fn stick_home(&self) -> Vec2 {
        self.dpad_home()
    }

    fn button_at(&self, p: Vec2) -> Option<TouchButton> {
        let dpad = self.dpad_home();
        let cell = self.dpad_cell();
        let dpad_left = dpad.x - 1.5 * cell;
        let dpad_top = dpad.y - 1.5 * cell;
        let dpad_size = 3.0 * cell;
        if p.x >= dpad_left
            && p.x <= dpad_left + dpad_size
            && p.y >= dpad_top
            && p.y <= dpad_top + dpad_size
        {
            let dx = p.x - dpad.x;
            let dy = p.y - dpad.y;
            if dx.abs() <= cell * 0.45 && dy.abs() <= cell * 0.45 {
                return Some(TouchButton::DpadCenter);
            }
            if dy.abs() >= dx.abs() {
                return if dy < 0.0 {
                    Some(TouchButton::Forward)
                } else {
                    Some(TouchButton::Backward)
                };
            } else {
                return if dx < 0.0 {
                    Some(TouchButton::Left)
                } else {
                    Some(TouchButton::Right)
                };
            }
        }

        for &btn in &TouchButton::ALL {
            if matches!(
                btn,
                TouchButton::Forward
                    | TouchButton::Backward
                    | TouchButton::Left
                    | TouchButton::Right
                    | TouchButton::DpadCenter
            ) {
                continue;
            }
            let (x, y, w, h) = self.button_rect(btn);
            let slop = 4.0 * self.gs;
            if p.x >= x - slop && p.x <= x + w + slop && p.y >= y - slop && p.y <= y + h + slop {
                return Some(btn);
            }
        }
        None
    }

    fn hotbar_slot_at(&self, p: Vec2) -> Option<u8> {
        crate::ui::hud::hotbar_slot_at(self.width, self.height, self.gs, (p.x, p.y))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TouchOutput {
    Look(Vec2),
    DestroyStart,
    DestroyEnd,
    DestroyCancel,
    Use,
    Tap(Action),
    Hotbar(u8),
    PointerMove(Vec2),
    PointerDown,
    PointerUp,
    RightClick,
    ToggleKeyboard,
}

enum Role {
    Stick {
        origin: Vec2,
        current: Vec2,
        radius: f32,
    },
    Look {
        start: Vec2,
        last: Vec2,
        since: Instant,
        still: bool,
        destroying: bool,
    },
    Jump,
    Forward,
    Backward,
    Left,
    Right,
    Primary,
    Secondary,
    PlayerList,
    VirtualMouseTrack {
        last: Vec2,
    },
    Pointer,
    Spent,
}

pub struct TouchControls {
    touches: HashMap<u64, Role>,
    seen: bool,
    sneak_locked: bool,
    virtual_mouse_mode: bool,
    virtual_cursor_pos: Vec2,
}

impl Default for TouchControls {
    fn default() -> Self {
        Self {
            touches: HashMap::new(),
            seen: false,
            sneak_locked: false,
            virtual_mouse_mode: false,
            virtual_cursor_pos: Vec2::new(400.0, 300.0),
        }
    }
}

impl TouchControls {
    pub fn seen(&self) -> bool {
        self.seen
    }

    pub fn is_sneak_locked(&self) -> bool {
        self.sneak_locked
    }

    pub fn is_virtual_mouse_mode(&self) -> bool {
        self.virtual_mouse_mode
    }

    pub fn virtual_cursor_position(&self) -> Vec2 {
        self.virtual_cursor_pos
    }

    pub fn on_touch(
        &mut self,
        id: u64,
        phase: TouchPhase,
        p: Vec2,
        gameplay: bool,
        layout: &TouchLayout,
        now: Instant,
    ) -> Vec<TouchOutput> {
        self.seen = true;
        match phase {
            TouchPhase::Started => self.start(id, p, gameplay, layout, now),
            TouchPhase::Moved => self.drag(id, p, layout),
            TouchPhase::Ended => self.end(id, now, true, gameplay),
            TouchPhase::Cancelled => self.end(id, now, false, gameplay),
        }
    }

    fn start(
        &mut self,
        id: u64,
        p: Vec2,
        gameplay: bool,
        layout: &TouchLayout,
        now: Instant,
    ) -> Vec<TouchOutput> {
        let (role, out) = if let Some(button) = layout.button_at(p) {
            match button {
                TouchButton::Keyboard => (Role::Spent, vec![TouchOutput::ToggleKeyboard]),
                TouchButton::MouseToggle => {
                    self.virtual_mouse_mode = !self.virtual_mouse_mode;
                    if self.virtual_mouse_mode {
                        self.virtual_cursor_pos =
                            Vec2::new(layout.width / 2.0, layout.height / 2.0);
                    }
                    (
                        Role::Spent,
                        vec![TouchOutput::PointerMove(self.virtual_cursor_pos)],
                    )
                }
                TouchButton::Primary if self.virtual_mouse_mode => {
                    (Role::Primary, vec![TouchOutput::PointerDown])
                }
                TouchButton::Secondary if self.virtual_mouse_mode => {
                    (Role::Secondary, vec![TouchOutput::RightClick])
                }
                _ if !gameplay => (
                    Role::Pointer,
                    vec![TouchOutput::PointerMove(p), TouchOutput::PointerDown],
                ),
                TouchButton::Forward => (Role::Forward, Vec::new()),
                TouchButton::Backward => (Role::Backward, Vec::new()),
                TouchButton::Left => (Role::Left, Vec::new()),
                TouchButton::Right => (Role::Right, Vec::new()),
                TouchButton::DpadCenter | TouchButton::Sneak => {
                    self.sneak_locked = !self.sneak_locked;
                    (Role::Spent, Vec::new())
                }
                TouchButton::Jump => (Role::Jump, Vec::new()),
                TouchButton::Primary => (Role::Primary, vec![TouchOutput::DestroyStart]),
                TouchButton::Secondary => (Role::Secondary, vec![TouchOutput::Use]),
                TouchButton::PlayerList => (Role::PlayerList, Vec::new()),
                other => match other.tapped_action() {
                    Some(action) => (Role::Spent, vec![TouchOutput::Tap(action)]),
                    None => (Role::Spent, Vec::new()),
                },
            }
        } else if !gameplay {
            if self.virtual_mouse_mode {
                (Role::VirtualMouseTrack { last: p }, Vec::new())
            } else {
                (
                    Role::Pointer,
                    vec![TouchOutput::PointerMove(p), TouchOutput::PointerDown],
                )
            }
        } else if let Some(slot) = layout.hotbar_slot_at(p) {
            (Role::Spent, vec![TouchOutput::Hotbar(slot)])
        } else if p.x < layout.width * STICK_ZONE {
            (
                Role::Stick {
                    origin: p,
                    current: p,
                    radius: STICK_RADIUS * layout.gs,
                },
                Vec::new(),
            )
        } else if self.virtual_mouse_mode {
            (Role::VirtualMouseTrack { last: p }, Vec::new())
        } else {
            (
                Role::Look {
                    start: p,
                    last: p,
                    since: now,
                    still: true,
                    destroying: false,
                },
                Vec::new(),
            )
        };
        self.touches.insert(id, role);
        out
    }

    fn drag(&mut self, id: u64, p: Vec2, layout: &TouchLayout) -> Vec<TouchOutput> {
        match self.touches.get_mut(&id) {
            Some(role @ (Role::Forward | Role::Backward | Role::Left | Role::Right)) => {
                if let Some(button) = layout.button_at(p) {
                    match button {
                        TouchButton::Forward => *role = Role::Forward,
                        TouchButton::Backward => *role = Role::Backward,
                        TouchButton::Left => *role = Role::Left,
                        TouchButton::Right => *role = Role::Right,
                        _ => {}
                    }
                }
                Vec::new()
            }
            Some(Role::Stick { current, .. }) => {
                *current = p;
                Vec::new()
            }
            Some(Role::VirtualMouseTrack { last }) => {
                let delta = p - *last;
                *last = p;
                self.virtual_cursor_pos = (self.virtual_cursor_pos + delta)
                    .clamp(Vec2::ZERO, Vec2::new(layout.width, layout.height));
                vec![TouchOutput::PointerMove(self.virtual_cursor_pos)]
            }
            Some(Role::Look {
                start, last, still, ..
            }) => {
                let delta = (p - *last) / layout.gs * LOOK_SCALE;
                *last = p;
                if p.distance(*start) > STILL_SLOP * layout.gs {
                    *still = false;
                }
                vec![TouchOutput::Look(delta)]
            }
            Some(Role::Pointer) => vec![TouchOutput::PointerMove(p)],
            _ => Vec::new(),
        }
    }

    fn end(&mut self, id: u64, now: Instant, lifted: bool, gameplay: bool) -> Vec<TouchOutput> {
        match self.touches.remove(&id) {
            Some(Role::Look {
                destroying: true, ..
            }) => vec![if gameplay {
                TouchOutput::DestroyEnd
            } else {
                TouchOutput::DestroyCancel
            }],
            Some(Role::Look { since, still, .. })
                if gameplay && lifted && still && now.duration_since(since) < TAP_TIME =>
            {
                vec![TouchOutput::Use]
            }
            Some(Role::Primary) => {
                if self.virtual_mouse_mode {
                    vec![TouchOutput::PointerUp]
                } else {
                    vec![if gameplay {
                        TouchOutput::DestroyEnd
                    } else {
                        TouchOutput::DestroyCancel
                    }]
                }
            }
            Some(Role::Pointer) => vec![TouchOutput::PointerUp],
            _ => Vec::new(),
        }
    }

    pub fn tick(&mut self, now: Instant, gameplay: bool) -> Vec<TouchOutput> {
        let mut out = Vec::new();
        if !gameplay {
            return out;
        }
        for role in self.touches.values_mut() {
            if let Role::Look {
                since,
                still,
                destroying,
                ..
            } = role
                && *still
                && !*destroying
                && now.duration_since(*since) >= HOLD_TIME
            {
                *destroying = true;
                out.push(TouchOutput::DestroyStart);
            }
        }
        out
    }

    pub fn stick_origin(&self) -> Option<Vec2> {
        self.touches.values().find_map(|role| match role {
            Role::Stick { origin, .. } => Some(*origin),
            _ => None,
        })
    }

    pub fn stick(&self) -> Option<Vec2> {
        let mut dpad = Vec2::ZERO;
        for role in self.touches.values() {
            match role {
                Role::Forward => dpad.y += 1.0,
                Role::Backward => dpad.y -= 1.0,
                Role::Left => dpad.x -= 1.0,
                Role::Right => dpad.x += 1.0,
                _ => {}
            }
        }
        if dpad != Vec2::ZERO {
            return Some(dpad.normalize_or_zero());
        }

        self.touches.values().find_map(|role| match role {
            Role::Stick {
                origin,
                current,
                radius,
            } => {
                let d = *current - *origin;
                if d == Vec2::ZERO {
                    Some(Vec2::ZERO)
                } else {
                    let len = d.length();
                    let clamped = d / len * len.min(*radius) / *radius;
                    Some(Vec2::new(clamped.x, -clamped.y))
                }
            }
            _ => None,
        })
    }

    pub fn is_forward_held(&self) -> bool {
        self.touches
            .values()
            .any(|role| matches!(role, Role::Forward))
    }

    pub fn is_backward_held(&self) -> bool {
        self.touches
            .values()
            .any(|role| matches!(role, Role::Backward))
    }

    pub fn is_left_held(&self) -> bool {
        self.touches.values().any(|role| matches!(role, Role::Left))
    }

    pub fn is_right_held(&self) -> bool {
        self.touches
            .values()
            .any(|role| matches!(role, Role::Right))
    }

    pub fn jump_held(&self) -> bool {
        self.touches.values().any(|role| matches!(role, Role::Jump))
    }

    pub fn is_primary_held(&self) -> bool {
        self.touches
            .values()
            .any(|role| matches!(role, Role::Primary))
    }

    pub fn is_secondary_held(&self) -> bool {
        self.touches
            .values()
            .any(|role| matches!(role, Role::Secondary))
    }

    pub fn player_list_held(&self) -> bool {
        self.touches
            .values()
            .any(|role| matches!(role, Role::PlayerList))
    }

    pub fn sneak_held(&self) -> bool {
        self.sneak_locked
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use glam::Vec2;
    use winit::event::TouchPhase;

    use super::{Action, TouchButton, TouchControls, TouchLayout, TouchOutput};

    const LAYOUT: TouchLayout = TouchLayout {
        width: 1600.0,
        height: 900.0,
        gs: 2.0,
    };
    const LOOK: Vec2 = Vec2::new(1000.0, 400.0);
    const STICK: Vec2 = Vec2::new(200.0, 600.0);

    fn ms(start: Instant, ms: u64) -> Instant {
        start + Duration::from_millis(ms)
    }

    fn touch(
        c: &mut TouchControls,
        id: u64,
        phase: TouchPhase,
        p: Vec2,
        at: Instant,
    ) -> Vec<TouchOutput> {
        c.on_touch(id, phase, p, true, &LAYOUT, at)
    }

    #[test]
    fn a_quick_still_tap_uses() {
        let (mut c, t) = (TouchControls::default(), Instant::now());
        assert!(touch(&mut c, 1, TouchPhase::Started, LOOK, t).is_empty());
        assert!(c.tick(ms(t, 100), true).is_empty());
        assert_eq!(
            touch(&mut c, 1, TouchPhase::Ended, LOOK, ms(t, 150)),
            [TouchOutput::Use]
        );
        assert!(c.seen());
    }

    #[test]
    fn a_look_touch_stays_out_of_a_screen_opened_under_it() {
        let (mut c, t) = (TouchControls::default(), Instant::now());
        touch(&mut c, 1, TouchPhase::Started, LOOK, t);
        assert!(c.tick(ms(t, 300), false).is_empty());
        assert!(
            c.on_touch(1, TouchPhase::Ended, LOOK, false, &LAYOUT, ms(t, 350))
                .is_empty()
        );

        touch(&mut c, 2, TouchPhase::Started, LOOK, t);
        assert_eq!(c.tick(ms(t, 300), true), [TouchOutput::DestroyStart]);
        assert_eq!(
            c.on_touch(2, TouchPhase::Ended, LOOK, false, &LAYOUT, ms(t, 400)),
            [TouchOutput::DestroyCancel]
        );
    }

    #[test]
    fn a_still_hold_destroys_until_lifted() {
        let (mut c, t) = (TouchControls::default(), Instant::now());
        touch(&mut c, 1, TouchPhase::Started, LOOK, t);
        assert_eq!(c.tick(ms(t, 300), true), [TouchOutput::DestroyStart]);
        assert!(c.tick(ms(t, 400), true).is_empty());
        assert_eq!(
            touch(&mut c, 1, TouchPhase::Ended, LOOK, ms(t, 500)),
            [TouchOutput::DestroyEnd]
        );
    }

    #[test]
    fn a_drag_looks_and_neither_uses_nor_destroys() {
        let (mut c, t) = (TouchControls::default(), Instant::now());
        touch(&mut c, 1, TouchPhase::Started, LOOK, t);
        let moved = LOOK + Vec2::new(40.0, -20.0);
        assert_eq!(
            touch(&mut c, 1, TouchPhase::Moved, moved, ms(t, 50)),
            [TouchOutput::Look(Vec2::new(50.0, -25.0))]
        );
        assert!(c.tick(ms(t, 400), true).is_empty());
        assert!(touch(&mut c, 1, TouchPhase::Ended, moved, ms(t, 100)).is_empty());
    }

    #[test]
    fn the_stick_clamps_to_unit_length_with_y_up() {
        let (mut c, t) = (TouchControls::default(), Instant::now());
        touch(&mut c, 2, TouchPhase::Started, STICK, t);
        touch(
            &mut c,
            2,
            TouchPhase::Moved,
            STICK + Vec2::new(0.0, -30.0),
            t,
        );
        assert_eq!(c.stick(), Some(Vec2::new(0.0, 0.5)));
        touch(
            &mut c,
            2,
            TouchPhase::Moved,
            STICK + Vec2::new(500.0, 0.0),
            t,
        );
        assert_eq!(c.stick(), Some(Vec2::new(1.0, 0.0)));
        touch(&mut c, 2, TouchPhase::Ended, STICK, t);
        assert_eq!(c.stick(), None);
    }

    #[test]
    fn move_and_look_at_once() {
        let (mut c, t) = (TouchControls::default(), Instant::now());
        touch(&mut c, 1, TouchPhase::Started, STICK, t);
        touch(&mut c, 2, TouchPhase::Started, LOOK, t);
        touch(
            &mut c,
            1,
            TouchPhase::Moved,
            STICK + Vec2::new(60.0, 0.0),
            t,
        );
        let look = touch(&mut c, 2, TouchPhase::Moved, LOOK + Vec2::new(8.0, 0.0), t);
        assert_eq!(look, [TouchOutput::Look(Vec2::new(10.0, 0.0))]);
        assert_eq!(c.stick(), Some(Vec2::new(1.0, 0.0)));
    }

    #[test]
    fn buttons_and_hotbar_fire_on_press() {
        let (mut c, t) = (TouchControls::default(), Instant::now());
        let pause = LAYOUT.button_centre(TouchButton::Pause);
        assert_eq!(
            touch(&mut c, 1, TouchPhase::Started, pause, t),
            [TouchOutput::Tap(Action::OpenMenu)]
        );
        assert!(touch(&mut c, 1, TouchPhase::Ended, pause, t).is_empty());
        assert_eq!(
            touch(
                &mut c,
                4,
                TouchPhase::Started,
                LAYOUT.button_centre(TouchButton::Chat),
                t
            ),
            [TouchOutput::Tap(Action::OpenChat)]
        );

        touch(
            &mut c,
            2,
            TouchPhase::Started,
            LAYOUT.button_centre(TouchButton::Jump),
            t,
        );
        assert!(c.jump_held());
        touch(&mut c, 2, TouchPhase::Ended, Vec2::ZERO, t);
        assert!(!c.jump_held());

        let kbd = LAYOUT.button_centre(TouchButton::Keyboard);
        assert_eq!(
            touch(&mut c, 5, TouchPhase::Started, kbd, t),
            [TouchOutput::ToggleKeyboard]
        );
        let pri = LAYOUT.button_centre(TouchButton::Primary);
        assert_eq!(
            touch(&mut c, 6, TouchPhase::Started, pri, t),
            [TouchOutput::DestroyStart]
        );
        assert_eq!(
            touch(&mut c, 6, TouchPhase::Ended, pri, t),
            [TouchOutput::DestroyEnd]
        );
        let sec = LAYOUT.button_centre(TouchButton::Secondary);
        assert_eq!(
            touch(&mut c, 7, TouchPhase::Started, sec, t),
            [TouchOutput::Use]
        );
        let snk = LAYOUT.button_centre(TouchButton::Sneak);
        assert!(!c.sneak_held());
        touch(&mut c, 8, TouchPhase::Started, snk, t);
        assert!(c.sneak_held());
        touch(&mut c, 9, TouchPhase::Started, snk, t);
        assert!(!c.sneak_held());

        let mouse_btn = LAYOUT.button_centre(TouchButton::MouseToggle);
        assert!(!c.is_virtual_mouse_mode());
        assert_eq!(
            touch(&mut c, 10, TouchPhase::Started, mouse_btn, t),
            [TouchOutput::PointerMove(Vec2::new(800.0, 450.0))]
        );
        assert!(c.is_virtual_mouse_mode());

        let first_slot = Vec2::new(LAYOUT.width / 2.0 - 91.0 * 2.0 + 10.0, LAYOUT.height - 10.0);
        assert_eq!(
            touch(&mut c, 3, TouchPhase::Started, first_slot, t),
            [TouchOutput::Hotbar(0)]
        );
    }

    #[test]
    fn outside_gameplay_a_touch_is_the_pointer() {
        let (mut c, t) = (TouchControls::default(), Instant::now());
        let p = Vec2::new(300.0, 200.0);
        assert_eq!(
            c.on_touch(1, TouchPhase::Started, p, false, &LAYOUT, t),
            [TouchOutput::PointerMove(p), TouchOutput::PointerDown]
        );
        assert_eq!(
            c.on_touch(1, TouchPhase::Ended, p, false, &LAYOUT, t),
            [TouchOutput::PointerUp]
        );
    }
}
