use std::collections::HashMap;
use std::sync::LazyLock;
use raylib::prelude::*;
pub static MOUSE_CURSOR_MAP : [MouseCursor; 9]  = [
	MouseCursor::MOUSE_CURSOR_ARROW,
	MouseCursor::MOUSE_CURSOR_IBEAM,

	MouseCursor::MOUSE_CURSOR_RESIZE_ALL,

	MouseCursor::MOUSE_CURSOR_RESIZE_NS,
	MouseCursor::MOUSE_CURSOR_RESIZE_EW,
	MouseCursor::MOUSE_CURSOR_RESIZE_NESW,
	MouseCursor::MOUSE_CURSOR_RESIZE_NWSE,

	MouseCursor::MOUSE_CURSOR_POINTING_HAND,
	MouseCursor::MOUSE_CURSOR_NOT_ALLOWED,
];

pub static KEYBOARD_MAP : LazyLock<HashMap<KeyboardKey, imgui::Key>> = LazyLock::new(|| {
	let mut map = HashMap::with_capacity(imgui::Key::COUNT);

	map.insert(KeyboardKey::KEY_APOSTROPHE, imgui::Key::Apostrophe);
	map.insert(KeyboardKey::KEY_COMMA, imgui::Key::Comma);
	map.insert(KeyboardKey::KEY_MINUS, imgui::Key::Minus);
	map.insert(KeyboardKey::KEY_PERIOD, imgui::Key::Period);
	map.insert(KeyboardKey::KEY_SLASH, imgui::Key::Slash);
	map.insert(KeyboardKey::KEY_ZERO, imgui::Key::Alpha0);
	map.insert(KeyboardKey::KEY_ONE, imgui::Key::Alpha1);
	map.insert(KeyboardKey::KEY_TWO, imgui::Key::Alpha2);
	map.insert(KeyboardKey::KEY_THREE, imgui::Key::Alpha3);
	map.insert(KeyboardKey::KEY_FOUR, imgui::Key::Alpha4);
	map.insert(KeyboardKey::KEY_FIVE, imgui::Key::Alpha5);
	map.insert(KeyboardKey::KEY_SIX, imgui::Key::Alpha6);
	map.insert(KeyboardKey::KEY_SEVEN, imgui::Key::Alpha7);
	map.insert(KeyboardKey::KEY_EIGHT, imgui::Key::Alpha8);
	map.insert(KeyboardKey::KEY_NINE, imgui::Key::Alpha9);
	map.insert(KeyboardKey::KEY_SEMICOLON, imgui::Key::Semicolon);
	map.insert(KeyboardKey::KEY_EQUAL, imgui::Key::Equal);
	map.insert(KeyboardKey::KEY_A, imgui::Key::A);
	map.insert(KeyboardKey::KEY_B, imgui::Key::B);
	map.insert(KeyboardKey::KEY_C, imgui::Key::C);
	map.insert(KeyboardKey::KEY_D, imgui::Key::D);
	map.insert(KeyboardKey::KEY_E, imgui::Key::E);
	map.insert(KeyboardKey::KEY_F, imgui::Key::F);
	map.insert(KeyboardKey::KEY_G, imgui::Key::G);
	map.insert(KeyboardKey::KEY_H, imgui::Key::H);
	map.insert(KeyboardKey::KEY_I, imgui::Key::I);
	map.insert(KeyboardKey::KEY_J, imgui::Key::J);
	map.insert(KeyboardKey::KEY_K, imgui::Key::K);
	map.insert(KeyboardKey::KEY_L, imgui::Key::L);
	map.insert(KeyboardKey::KEY_M, imgui::Key::M);
	map.insert(KeyboardKey::KEY_N, imgui::Key::N);
	map.insert(KeyboardKey::KEY_O, imgui::Key::O);
	map.insert(KeyboardKey::KEY_P, imgui::Key::P);
	map.insert(KeyboardKey::KEY_Q, imgui::Key::Q);
	map.insert(KeyboardKey::KEY_R, imgui::Key::R);
	map.insert(KeyboardKey::KEY_S, imgui::Key::S);
	map.insert(KeyboardKey::KEY_T, imgui::Key::T);
	map.insert(KeyboardKey::KEY_U, imgui::Key::U);
	map.insert(KeyboardKey::KEY_V, imgui::Key::V);
	map.insert(KeyboardKey::KEY_W, imgui::Key::W);
	map.insert(KeyboardKey::KEY_X, imgui::Key::X);
	map.insert(KeyboardKey::KEY_Y, imgui::Key::Y);
	map.insert(KeyboardKey::KEY_Z, imgui::Key::Z);
	map.insert(KeyboardKey::KEY_SPACE, imgui::Key::Space);
	map.insert(KeyboardKey::KEY_ESCAPE, imgui::Key::Escape);
	map.insert(KeyboardKey::KEY_ENTER, imgui::Key::Enter);
	map.insert(KeyboardKey::KEY_TAB, imgui::Key::Tab);
	map.insert(KeyboardKey::KEY_BACKSPACE, imgui::Key::Backspace);
	map.insert(KeyboardKey::KEY_INSERT, imgui::Key::Insert);
	map.insert(KeyboardKey::KEY_DELETE, imgui::Key::Delete);
	map.insert(KeyboardKey::KEY_RIGHT, imgui::Key::RightArrow);
	map.insert(KeyboardKey::KEY_LEFT, imgui::Key::LeftArrow);
	map.insert(KeyboardKey::KEY_DOWN, imgui::Key::DownArrow);
	map.insert(KeyboardKey::KEY_UP, imgui::Key::UpArrow);
	map.insert(KeyboardKey::KEY_PAGE_UP, imgui::Key::PageUp);
	map.insert(KeyboardKey::KEY_PAGE_DOWN, imgui::Key::PageDown);
	map.insert(KeyboardKey::KEY_HOME, imgui::Key::Home);
	map.insert(KeyboardKey::KEY_END, imgui::Key::End);
	map.insert(KeyboardKey::KEY_CAPS_LOCK, imgui::Key::CapsLock);
	map.insert(KeyboardKey::KEY_SCROLL_LOCK, imgui::Key::ScrollLock);
	map.insert(KeyboardKey::KEY_NUM_LOCK, imgui::Key::NumLock);
	map.insert(KeyboardKey::KEY_PRINT_SCREEN, imgui::Key::PrintScreen);
	map.insert(KeyboardKey::KEY_PAUSE, imgui::Key::Pause);
	map.insert(KeyboardKey::KEY_F1, imgui::Key::F1);
	map.insert(KeyboardKey::KEY_F2, imgui::Key::F2);
	map.insert(KeyboardKey::KEY_F3, imgui::Key::F3);
	map.insert(KeyboardKey::KEY_F4, imgui::Key::F4);
	map.insert(KeyboardKey::KEY_F5, imgui::Key::F5);
	map.insert(KeyboardKey::KEY_F6, imgui::Key::F6);
	map.insert(KeyboardKey::KEY_F7, imgui::Key::F7);
	map.insert(KeyboardKey::KEY_F8, imgui::Key::F8);
	map.insert(KeyboardKey::KEY_F9, imgui::Key::F9);
	map.insert(KeyboardKey::KEY_F10, imgui::Key::F10);
	map.insert(KeyboardKey::KEY_F11, imgui::Key::F11);
	map.insert(KeyboardKey::KEY_F12, imgui::Key::F12);
	map.insert(KeyboardKey::KEY_LEFT_SHIFT, imgui::Key::LeftShift);
	map.insert(KeyboardKey::KEY_LEFT_CONTROL, imgui::Key::LeftCtrl);
	map.insert(KeyboardKey::KEY_LEFT_ALT, imgui::Key::LeftAlt);
	map.insert(KeyboardKey::KEY_LEFT_SUPER, imgui::Key::LeftSuper);
	map.insert(KeyboardKey::KEY_RIGHT_SHIFT, imgui::Key::RightShift);
	map.insert(KeyboardKey::KEY_RIGHT_CONTROL, imgui::Key::RightCtrl);
	map.insert(KeyboardKey::KEY_RIGHT_ALT, imgui::Key::RightAlt);
	map.insert(KeyboardKey::KEY_RIGHT_SUPER, imgui::Key::RightSuper);
	map.insert(KeyboardKey::KEY_KB_MENU, imgui::Key::Menu);
	map.insert(KeyboardKey::KEY_LEFT_BRACKET, imgui::Key::LeftBracket);
	map.insert(KeyboardKey::KEY_BACKSLASH, imgui::Key::Backslash);
	map.insert(KeyboardKey::KEY_RIGHT_BRACKET, imgui::Key::RightBracket);
	map.insert(KeyboardKey::KEY_GRAVE, imgui::Key::GraveAccent);
	map.insert(KeyboardKey::KEY_KP_0, imgui::Key::Keypad0);
	map.insert(KeyboardKey::KEY_KP_1, imgui::Key::Keypad1);
	map.insert(KeyboardKey::KEY_KP_2, imgui::Key::Keypad2);
	map.insert(KeyboardKey::KEY_KP_3, imgui::Key::Keypad3);
	map.insert(KeyboardKey::KEY_KP_4, imgui::Key::Keypad4);
	map.insert(KeyboardKey::KEY_KP_5, imgui::Key::Keypad5);
	map.insert(KeyboardKey::KEY_KP_6, imgui::Key::Keypad6);
	map.insert(KeyboardKey::KEY_KP_7, imgui::Key::Keypad7);
	map.insert(KeyboardKey::KEY_KP_8, imgui::Key::Keypad8);
	map.insert(KeyboardKey::KEY_KP_9, imgui::Key::Keypad9);
	map.insert(KeyboardKey::KEY_KP_DECIMAL, imgui::Key::KeypadDecimal);
	map.insert(KeyboardKey::KEY_KP_DIVIDE, imgui::Key::KeypadDivide);
	map.insert(KeyboardKey::KEY_KP_MULTIPLY, imgui::Key::KeypadMultiply);
	map.insert(KeyboardKey::KEY_KP_SUBTRACT, imgui::Key::KeypadSubtract);
	map.insert(KeyboardKey::KEY_KP_ADD, imgui::Key::KeypadAdd);
	map.insert(KeyboardKey::KEY_KP_ENTER, imgui::Key::KeypadEnter);
	map.insert(KeyboardKey::KEY_KP_EQUAL, imgui::Key::KeypadEqual);

	map
});