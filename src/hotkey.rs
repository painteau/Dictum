use rdev::{listen, EventType, Key};
use crossbeam_channel::Sender;
use crate::{AppEvent, config::Config};

fn key_from_str(s: &str) -> Key {
    match s.to_uppercase().as_str() {
        "F1"  => Key::F1,
        "F2"  => Key::F2,
        "F3"  => Key::F3,
        "F4"  => Key::F4,
        "F5"  => Key::F5,
        "F6"  => Key::F6,
        "F7"  => Key::F7,
        "F8"  => Key::F8,
        "F9"  => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        "SPACE"       => Key::Space,
        "TAB"         => Key::Tab,
        "INSERT"      => Key::Insert,
        "HOME"        => Key::Home,
        "END"         => Key::End,
        "PAGEUP"      => Key::PageUp,
        "PAGEDOWN"    => Key::PageDown,
        "SCROLLLOCK"  => Key::ScrollLock,
        "PAUSE"       => Key::Pause,
        "CAPSLOCK"    => Key::CapsLock,
        "BACKQUOTE" | "BACKTICK" => Key::BackQuote,
        "NUM0" => Key::Num0, "NUM1" => Key::Num1, "NUM2" => Key::Num2,
        "NUM3" => Key::Num3, "NUM4" => Key::Num4, "NUM5" => Key::Num5,
        "NUM6" => Key::Num6, "NUM7" => Key::Num7, "NUM8" => Key::Num8,
        "NUM9" => Key::Num9,
        "BACKSPACE" => Key::Backspace,
        "DELETE" | "DEL" => Key::Delete,
        "ENTER" | "RETURN" => Key::Return,
        "ESCAPE" | "ESC" => Key::Escape,
        "NUMLOCK" => Key::NumLock,
        "KPMINUS" | "NUMPADMINUS" => Key::KpMinus,
        "KPPLUS" | "NUMPADPLUS" => Key::KpPlus,
        "KPMULTIPLY" | "NUMPADMULTIPLY" => Key::KpMultiply,
        "KPDIVIDE" | "NUMPADDIVIDE" => Key::KpDivide,
        "KPRETURN" | "NUMPADENTER" => Key::KpReturn,
        _ => {
            log::warn!("Touche '{}' inconnue, fallback F9. Valides : F1-F12, Space, Insert, Home, End, PageUp, PageDown, ScrollLock, Pause, CapsLock", s);
            Key::F9
        }
    }
}

/// Blocking — call in a dedicated thread.
pub fn start(config: Config, tx: Sender<AppEvent>) {
    let combo = format!(
        "{}{}{}{}",
        if config.hotkey.ctrl  { "Ctrl+" } else { "" },
        if config.hotkey.alt   { "Alt+"  } else { "" },
        if config.hotkey.shift { "Shift+"} else { "" },
        config.hotkey.key
    );
    log::info!("Hotkey active : {} (maintenir pour dicter)", combo);
    let target = key_from_str(&config.hotkey.key);
    let need_ctrl  = config.hotkey.ctrl;
    let need_alt   = config.hotkey.alt;
    let need_shift = config.hotkey.shift;

    let mut ctrl_down  = false;
    let mut alt_down   = false;
    let mut shift_down = false;
    let mut active     = false;

    if let Err(e) = listen(move |event| {
        match event.event_type {
            EventType::KeyPress(Key::ControlLeft) | EventType::KeyPress(Key::ControlRight) => ctrl_down = true,
            EventType::KeyRelease(Key::ControlLeft) | EventType::KeyRelease(Key::ControlRight) => ctrl_down = false,
            EventType::KeyPress(Key::Alt) | EventType::KeyPress(Key::AltGr) => alt_down = true,
            EventType::KeyRelease(Key::Alt) | EventType::KeyRelease(Key::AltGr) => alt_down = false,
            EventType::KeyPress(Key::ShiftLeft) | EventType::KeyPress(Key::ShiftRight) => shift_down = true,
            EventType::KeyRelease(Key::ShiftLeft) | EventType::KeyRelease(Key::ShiftRight) => shift_down = false,

            EventType::KeyPress(ref k) if *k == target => {
                let mods_ok = (!need_ctrl || ctrl_down)
                    && (!need_alt || alt_down)
                    && (!need_shift || shift_down);
                if !mods_ok {
                    log::debug!("Hotkey {} pressée sans les modificateurs requis", combo);
                }
                if mods_ok && !active {
                    active = true;
                    log::info!("Hotkey pressée → démarrage enregistrement");
                    let _ = tx.send(AppEvent::RecordStart);
                }
            }
            EventType::KeyRelease(ref k) if *k == target => {
                if active {
                    active = false;
                    log::info!("Hotkey relâchée → arrêt enregistrement");
                    let _ = tx.send(AppEvent::RecordStop);
                }
            }
            _ => {}
        }
    }) {
        log::error!("Erreur listener hotkey : {e:?}");
    }
}
