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
pub const BUTTON_RADIUS: f32 = 14.0;

#[derive(Clone, Copy)]
pub struct TouchLayout {
    pub width: f32,
    pub height: f32,
    pub gs: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchButton {
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
}

impl TouchButton {
    pub const ALL: [Self; 11] = [
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
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Jump => "Jump",
            Self::Chat => "Chat",
            Self::Inventory => "Inv",
            Self::Pause => "| |",
            Self::Keyboard => "Kbd",
            Self::Debug => "F3",
            Self::ThirdPerson => "F5",
            Self::PlayerList => "Tab",
            Self::Sneak => "Snk",
            Self::Primary => "PRI",
            Self::Secondary => "SEC",
        }
    }

    fn tapped_action(self) -> Option<Action> {
        match self {
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
        }
    }
}

impl TouchLayout {
    pub fn button_centre(&self, button: TouchButton) -> Vec2 {
        let inset = 2.0 * BUTTON_RADIUS * self.gs;
        match button {
            TouchButton::Jump => Vec2::new(self.width - inset, self.height - 1.5 * inset),
            TouchButton::Secondary => Vec2::new(self.width - inset, self.height - 3.0 * inset),
            TouchButton::Primary => Vec2::new(self.width - 2.5 * inset, self.height - 2.25 * inset),
            TouchButton::Pause => Vec2::new(self.width - inset, inset),
            TouchButton::Inventory => Vec2::new(self.width - 2.5 * inset, inset),
            TouchButton::Chat => Vec2::new(self.width - 4.0 * inset, inset),
            TouchButton::Keyboard => Vec2::new(self.width - 5.5 * inset, inset),
            TouchButton::ThirdPerson => Vec2::new(self.width - 7.0 * inset, inset),
            TouchButton::Debug => Vec2::new(inset, inset),
            TouchButton::PlayerList => Vec2::new(2.5 * inset, inset),
            TouchButton::Sneak => Vec2::new(self.width - 4.0 * inset, self.height - 1.5 * inset),
        }
    }

    pub fn stick_home(&self) -> Vec2 {
        let inset = 1.6 * STICK_RADIUS * self.gs;
        Vec2::new(inset, self.height - inset)
    }

    fn button_at(&self, p: Vec2) -> Option<TouchButton> {
        TouchButton::ALL
            .into_iter()
            .find(|&b| p.distance(self.button_centre(b)) <= BUTTON_RADIUS * self.gs)
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
    Primary,
    Secondary,
    PlayerList,
    Pointer,
    Spent,
}

#[derive(Default)]
pub struct TouchControls {
    touches: HashMap<u64, Role>,
    seen: bool,
    sneak_locked: bool,
}

impl TouchControls {
    pub fn seen(&self) -> bool {
        self.seen
    }

    pub fn is_sneak_locked(&self) -> bool {
        self.sneak_locked
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
        let (role, out) = if !gameplay {
            (
                Role::Pointer,
                vec![TouchOutput::PointerMove(p), TouchOutput::PointerDown],
            )
        } else if let Some(button) = layout.button_at(p) {
            match button {
                TouchButton::Jump => (Role::Jump, Vec::new()),
                TouchButton::Primary => (Role::Primary, vec![TouchOutput::DestroyStart]),
                TouchButton::Secondary => (Role::Secondary, vec![TouchOutput::Use]),
                TouchButton::PlayerList => (Role::PlayerList, Vec::new()),
                TouchButton::Sneak => {
                    self.sneak_locked = !self.sneak_locked;
                    (Role::Spent, Vec::new())
                }
                TouchButton::Keyboard => (Role::Spent, vec![TouchOutput::ToggleKeyboard]),
                other => match other.tapped_action() {
                    Some(action) => (Role::Spent, vec![TouchOutput::Tap(action)]),
                    None => (Role::Spent, Vec::new()),
                },
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
            Some(Role::Stick { current, .. }) => {
                *current = p;
                Vec::new()
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
            Some(Role::Primary) => vec![if gameplay {
                TouchOutput::DestroyEnd
            } else {
                TouchOutput::DestroyCancel
            }],
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

    pub fn jump_held(&self) -> bool {
        self.touches.values().any(|role| matches!(role, Role::Jump))
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
        width: 800.0,
        height: 600.0,
        gs: 2.0,
    };
    const STICK: Vec2 = Vec2::new(100.0, 500.0);
    const LOOK: Vec2 = Vec2::new(500.0, 300.0);

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
