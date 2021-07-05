use cgmath::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, KeyboardEvent, MouseEvent};

// TODO: can Clone be removed for these types?
/// An event.
#[derive(Clone, Debug)]
pub enum Event {
    KeyDown(Key),
    KeyUp(Key),
    CharEntered(char),
    MouseDown(MouseButton, Point2<i32>),
    MouseUp(MouseButton, Point2<i32>),
    MouseMove {
        pos: Point2<i32>,
        movement: Vector2<i32>,
    },
    MouseEnter,
    MouseLeave,
    FocusGained,
    FocusLost,
    /// When this is received, apps should call something like `self.screen_surface.set_size(&self.context, new_size);`
    // TODO: do this automatically
    WindowResized(Vector2<u32>),
    PointerLocked,
    PointerUnlocked,
    Scroll(f64),
}

pub type Keycode = String;

/// A key.
#[derive(Clone, Debug)]
pub struct Key {
    /// These correspond to `event.code` values.
    /// On desktop, an attempt is made to convert from GLFW keycodes to JS `event.code` values.
    pub code: String,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub is_modifier: bool,
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn char_from_js(js_key: &KeyboardEvent) -> Option<char> {
    // TODO: find a better way to check if the char is printable
    let key = js_key.key();
    if key.len() == 1 {
        Some(key.chars().next().unwrap())
    } else {
        None
    }
}

impl Key {
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn from_js(js_key: &KeyboardEvent) -> Self {
        Self {
            code: js_key.code(),
            shift: js_key.shift_key(),
            ctrl: js_key.ctrl_key(),
            alt: js_key.alt_key(),
            is_modifier: js_key.key() == "Shift"
                || js_key.key() == "Control"
                || js_key.key() == "Alt",
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn from_glfw(key: glfw::Key, modifiers: glfw::Modifiers) -> Option<Self> {
        use glfw::Key::*;
        let code = match key {
            Space => Some("Space"),
            Apostrophe => Some("Quote"),
            Comma => Some("Comma"),
            Minus => Some("Minus"),
            Period => Some("Period"),
            Slash => Some("Slash"),
            Num0 => Some("Digit0"),
            Num1 => Some("Digit1"),
            Num2 => Some("Digit2"),
            Num3 => Some("Digit3"),
            Num4 => Some("Digit4"),
            Num5 => Some("Digit5"),
            Num6 => Some("Digit6"),
            Num7 => Some("Digit7"),
            Num8 => Some("Digit8"),
            Num9 => Some("Digit9"),
            Semicolon => Some("Semicolon"),
            Equal => Some("Equal"),
            A => Some("KeyA"),
            B => Some("KeyB"),
            C => Some("KeyC"),
            D => Some("KeyD"),
            E => Some("KeyE"),
            F => Some("KeyF"),
            G => Some("KeyG"),
            H => Some("KeyH"),
            I => Some("KeyI"),
            J => Some("KeyJ"),
            K => Some("KeyK"),
            L => Some("KeyL"),
            M => Some("KeyM"),
            N => Some("KeyN"),
            O => Some("KeyO"),
            P => Some("KeyP"),
            Q => Some("KeyQ"),
            R => Some("KeyR"),
            S => Some("KeyS"),
            T => Some("KeyT"),
            U => Some("KeyU"),
            V => Some("KeyV"),
            W => Some("KeyW"),
            X => Some("KeyX"),
            Y => Some("KeyY"),
            Z => Some("KeyZ"),
            LeftBracket => Some("BracketLeft"),
            Backslash => Some("Backslash"),
            RightBracket => Some("BracketRight"),
            GraveAccent => Some("Backquote"),
            Escape => Some("Escape"),
            Enter => Some("Enter"),
            Tab => Some("Tab"),
            Backspace => Some("Backspace"),
            Insert => Some("Insert"),
            Delete => Some("Delete"),
            Right => Some("ArrowRight"),
            Left => Some("ArrowLeft"),
            Down => Some("ArrowDown"),
            Up => Some("ArrowUp"),
            PageUp => Some("PageUp"),
            PageDown => Some("PageDown"),
            Home => Some("Home"),
            End => Some("End"),
            CapsLock => Some("CapsLock"),
            ScrollLock => Some("ScrollLock"),
            NumLock => Some("NumLock"),
            F1 => Some("F1"),
            F2 => Some("F2"),
            F3 => Some("F3"),
            F4 => Some("F4"),
            F5 => Some("F5"),
            F6 => Some("F6"),
            F7 => Some("F7"),
            F8 => Some("F8"),
            F9 => Some("F9"),
            F10 => Some("F10"),
            F11 => Some("F11"),
            F12 => Some("F12"),
            Kp0 => Some("Numpad0"),
            Kp1 => Some("Numpad1"),
            Kp2 => Some("Numpad2"),
            Kp3 => Some("Numpad3"),
            Kp4 => Some("Numpad4"),
            Kp5 => Some("Numpad5"),
            Kp6 => Some("Numpad6"),
            Kp7 => Some("Numpad7"),
            Kp8 => Some("Numpad8"),
            Kp9 => Some("Numpad9"),
            // Other keys aren't yet supported; if you need other keys, please file an issue or send a PR
            _ => None,
        };
        if let Some(code) = code {
            Some(Self {
                code: code.to_owned(),
                shift: modifiers.contains(glfw::Modifiers::Shift),
                ctrl: modifiers.contains(glfw::Modifiers::Control),
                alt: modifiers.contains(glfw::Modifiers::Alt),
                is_modifier: key == LeftShift
                    || key == LeftControl
                    || key == LeftAlt
                    || key == RightShift
                    || key == RightControl
                    || key == RightAlt,
            })
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

impl MouseButton {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn from_glfw(button: glfw::MouseButton) -> Option<Self> {
        match button {
            glfw::MouseButton::Button1 => Some(MouseButton::Left),
            glfw::MouseButton::Button2 => Some(MouseButton::Right),
            glfw::MouseButton::Button3 => Some(MouseButton::Middle),
            _ => None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn from_js(js_button: i16) -> Option<Self> {
        match js_button {
            0 => Some(MouseButton::Left),
            1 => Some(MouseButton::Middle),
            2 => Some(MouseButton::Right),
            3 => Some(MouseButton::Back),
            4 => Some(MouseButton::Forward),
            _ => None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn mouse_pos_from_js(event: MouseEvent) -> Point2<i32> {
    point2(event.offset_x(), event.offset_y())
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn mouse_down_event_from_js(event: MouseEvent) -> Option<Event> {
    let button = MouseButton::from_js(event.button())?;
    Some(Event::MouseDown(button, mouse_pos_from_js(event)))
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn mouse_up_event_from_js(event: MouseEvent) -> Option<Event> {
    let button = MouseButton::from_js(event.button())?;
    Some(Event::MouseUp(button, mouse_pos_from_js(event)))
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn mouse_move_event_from_js(event: MouseEvent) -> Option<Event> {
    Some(Event::MouseMove {
        movement: vec2(event.movement_x(), event.movement_y()),
        pos: mouse_pos_from_js(event),
    })
}

#[cfg(target_arch = "wasm32")]
pub fn get_window_size() -> Vector2<u32> {
    let window = window().unwrap();
    vec2(
        window.inner_width().unwrap().as_f64().unwrap() as u32,
        window.inner_height().unwrap().as_f64().unwrap() as u32,
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub fn event_from_glfw(
    event: &glfw::WindowEvent,
    window: &glfw::Window,
    prev_cursor_pos: &mut Option<Point2<i32>>,
) -> Option<Event> {
    match *event {
        glfw::WindowEvent::MouseButton(button, action, _) => {
            let (cursor_x, cursor_y) = window.get_cursor_pos();
            let cursor_pos = point2(cursor_x as i32, cursor_y as i32);
            if action == glfw::Action::Release {
                Some(Event::MouseUp(MouseButton::from_glfw(button)?, cursor_pos))
            } else {
                Some(Event::MouseDown(MouseButton::from_glfw(button)?, cursor_pos))
            }
        }
        glfw::WindowEvent::CursorPos(cursor_x, cursor_y) => {
            let cursor_pos = point2(cursor_x as i32, cursor_y as i32);
            let res = if let Some(prev_cursor_pos) = prev_cursor_pos {
                let movement = cursor_pos - *prev_cursor_pos;
                Some(Event::MouseMove { pos: cursor_pos, movement })
            } else {
                // TODO: send initial cursor position, but without movement info
                None
            };
            *prev_cursor_pos = Some(cursor_pos);
            res
        }
        glfw::WindowEvent::Key(key, _, action, modifiers) => {
            let key = Key::from_glfw(key, modifiers)?;
            if action == glfw::Action::Release {
                Some(Event::KeyUp(key))
            } else {
                Some(Event::KeyDown(key))
            }
        }
        glfw::WindowEvent::CursorEnter(true) => Some(Event::MouseEnter),
        glfw::WindowEvent::CursorEnter(false) => Some(Event::MouseLeave),
        glfw::WindowEvent::FramebufferSize(width, height) => {
            Some(Event::WindowResized(vec2(width as u32, height as u32)))
        }
        glfw::WindowEvent::Scroll(_x, y) => Some(Event::Scroll(-y.signum())),
        glfw::WindowEvent::Focus(true) => Some(Event::FocusGained),
        glfw::WindowEvent::Focus(false) => Some(Event::FocusLost),
        _ => None,
    }
}
