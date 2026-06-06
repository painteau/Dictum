/// Envoie la touche VK_MEDIA_PLAY_PAUSE (toggle pause/reprise).
pub fn toggle_media() {
    #[cfg(windows)]
    unsafe {
        use winapi::um::winuser::{SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP};
        const VK_MEDIA_PLAY_PAUSE: u16 = 0xB3;

        let make_input = |flags: u32| -> INPUT {
            let mut input: INPUT = std::mem::zeroed();
            input.type_ = INPUT_KEYBOARD;
            *input.u.ki_mut() = KEYBDINPUT {
                wVk: VK_MEDIA_PLAY_PAUSE,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            };
            input
        };

        let mut inputs = [make_input(0), make_input(KEYEVENTF_KEYUP)];
        SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}
