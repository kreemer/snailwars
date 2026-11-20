//! Mouse, keyboard, and gamepad input.
//!
//! The gamepad's left stick controls a virtual mouse cursor so all screens
//! continue to use the same UI and world hit-testing as mouse input.

use gilrs::{Axis, Button, Gilrs};
use macroquad::prelude::*;

const CURSOR_SPEED: f32 = 900.0;
const STICK_DEADZONE: f32 = 0.18;

#[derive(Clone, Copy, Default)]
struct Buttons {
    south: bool,
    east: bool,
    west: bool,
    north: bool,
    left_trigger: bool,
    right_trigger: bool,
}

#[derive(Default)]
struct GamepadState {
    left_stick: Vec2,
    right_stick: Vec2,
    buttons: Buttons,
}

#[derive(Clone, Copy)]
pub struct Controls {
    pub screen_mouse: Vec2,
    pub clicked: bool,
    pub cancelled: bool,
    pub speed_pressed: bool,
    pub restart_pressed: bool,
    pub camera_pan: Vec2,
    pub zoom_steps: f32,
    pub gamepad_cursor: Option<Vec2>,
}

pub struct Input {
    gilrs: Option<Gilrs>,
    cursor: Vec2,
    gamepad_mode: bool,
    previous_buttons: Buttons,
}

impl Input {
    pub fn new() -> Self {
        let gilrs = match Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(error) => {
                eprintln!("gamepad input unavailable: {error}");
                None
            }
        };
        let cursor = Vec2::from(mouse_position());

        Input {
            gilrs,
            cursor,
            gamepad_mode: false,
            previous_buttons: Buttons::default(),
        }
    }

    pub fn update(&mut self, dt: f32) -> Controls {
        let mouse = Vec2::from(mouse_position());
        let mouse_moved = mouse_delta_position().length_squared() > 0.25;
        let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);
        let state = self.read_gamepad();
        let pressed = ButtonPresses::between(self.previous_buttons, state.buttons);
        self.previous_buttons = state.buttons;

        let gamepad_active = state.left_stick.length_squared() > 0.0
            || state.right_stick.length_squared() > 0.0
            || pressed.any();
        if mouse_moved || mouse_clicked {
            self.gamepad_mode = false;
            self.cursor = mouse;
        } else if gamepad_active {
            if !self.gamepad_mode {
                self.cursor = mouse;
            }
            self.gamepad_mode = true;
        }

        if self.gamepad_mode {
            self.cursor += state.left_stick * CURSOR_SPEED * dt;
            self.cursor.x = self.cursor.x.clamp(0.0, screen_width());
            self.cursor.y = self.cursor.y.clamp(0.0, screen_height());
        }
        show_mouse(!self.gamepad_mode);

        let gamepad_cursor = self.gamepad_mode.then_some(self.cursor);
        Controls {
            screen_mouse: gamepad_cursor.unwrap_or(mouse),
            clicked: mouse_clicked || (self.gamepad_mode && pressed.south),
            cancelled: self.gamepad_mode && pressed.east,
            speed_pressed: self.gamepad_mode && pressed.north,
            restart_pressed: self.gamepad_mode && pressed.west,
            camera_pan: state.right_stick,
            zoom_steps: if self.gamepad_mode {
                f32::from(pressed.right_trigger) - f32::from(pressed.left_trigger)
            } else {
                0.0
            },
            gamepad_cursor,
        }
    }

    fn read_gamepad(&mut self) -> GamepadState {
        let Some(gilrs) = &mut self.gilrs else {
            return GamepadState::default();
        };

        while gilrs.next_event().is_some() {}
        let state = gilrs
            .gamepads()
            .next()
            .map(|(_, gamepad)| GamepadState {
                left_stick: vec2(
                    apply_deadzone(gamepad.value(Axis::LeftStickX)),
                    apply_deadzone(gamepad.value(Axis::LeftStickY)),
                ),
                right_stick: vec2(
                    apply_deadzone(gamepad.value(Axis::RightStickX)),
                    apply_deadzone(gamepad.value(Axis::RightStickY)),
                ),
                buttons: Buttons {
                    south: gamepad.is_pressed(Button::South),
                    east: gamepad.is_pressed(Button::East),
                    west: gamepad.is_pressed(Button::West),
                    north: gamepad.is_pressed(Button::North),
                    left_trigger: gamepad.is_pressed(Button::LeftTrigger),
                    right_trigger: gamepad.is_pressed(Button::RightTrigger),
                },
            })
            .unwrap_or_default();
        gilrs.inc();
        state
    }
}

struct ButtonPresses {
    south: bool,
    east: bool,
    west: bool,
    north: bool,
    left_trigger: bool,
    right_trigger: bool,
}

impl ButtonPresses {
    fn between(previous: Buttons, current: Buttons) -> Self {
        ButtonPresses {
            south: current.south && !previous.south,
            east: current.east && !previous.east,
            west: current.west && !previous.west,
            north: current.north && !previous.north,
            left_trigger: current.left_trigger && !previous.left_trigger,
            right_trigger: current.right_trigger && !previous.right_trigger,
        }
    }

    fn any(&self) -> bool {
        self.south
            || self.east
            || self.west
            || self.north
            || self.left_trigger
            || self.right_trigger
    }
}

fn apply_deadzone(value: f32) -> f32 {
    let magnitude = value.abs();
    if magnitude <= STICK_DEADZONE {
        0.0
    } else {
        value.signum() * (magnitude - STICK_DEADZONE) / (1.0 - STICK_DEADZONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stick_deadzone_filters_drift_and_rescales_motion() {
        assert_eq!(apply_deadzone(0.1), 0.0);
        assert!((apply_deadzone(1.0) - 1.0).abs() < f32::EPSILON);
        assert!((apply_deadzone(-0.5) + 0.3902439).abs() < 0.0001);
    }

    #[test]
    fn button_press_is_reported_only_on_its_rising_edge() {
        let previous = Buttons::default();
        let pressed = Buttons {
            south: true,
            ..Buttons::default()
        };
        assert!(ButtonPresses::between(previous, pressed).south);
        assert!(!ButtonPresses::between(pressed, pressed).south);
    }
}
