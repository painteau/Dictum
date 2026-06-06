/// Envoie la touche VK_MEDIA_PLAY_PAUSE (toggle pause/reprise).
pub fn toggle_media() {
    #[cfg(windows)]
    unsafe {
        use winapi::um::winuser::{SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP};
        const VK_MEDIA_PLAY_PAUSE: u16 = 0xB3;

        let mut inputs = [
            INPUT {
                type_: INPUT_KEYBOARD,
                u: unsafe {
                    let mut u = std::mem::zeroed::<winapi::um::winuser::INPUT_u>();
                    *u.ki_mut() = KEYBDINPUT {
                        wVk: VK_MEDIA_PLAY_PAUSE,
                        wScan: 0,
                        dwFlags: 0,
                        time: 0,
                        dwExtraInfo: 0,
                    };
                    u
                },
            },
            INPUT {
                type_: INPUT_KEYBOARD,
                u: unsafe {
                    let mut u = std::mem::zeroed::<winapi::um::winuser::INPUT_u>();
                    *u.ki_mut() = KEYBDINPUT {
                        wVk: VK_MEDIA_PLAY_PAUSE,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    };
                    u
                },
            },
        ];
        SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}
