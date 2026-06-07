/// Notification ballon Windows via Shell_NotifyIconW.
/// Utilisé pour signaler une mise à jour disponible sans dialog bloquant.

/// Affiche une notification toast non bloquante via PowerShell.
/// Fallback léger sans dépendance WinRT supplémentaire.
pub fn show_toast(title: &str, message: &str) {
    let title = title.replace('"', "'");
    let message = message.replace('"', "'");

    let script = format!(
        r#"Add-Type -AssemblyName System.Windows.Forms; \
$n = New-Object System.Windows.Forms.NotifyIcon; \
$n.Icon = [System.Drawing.Icon]::ExtractAssociatedIcon((Get-Process -Id $PID).Path); \
$n.Visible = $true; \
$n.ShowBalloonTip(4000, "{}", "{}", [System.Windows.Forms.ToolTipIcon]::Info); \
Start-Sleep -Milliseconds 4500; \
$n.Dispose()"#,
        title, message
    );

    std::thread::spawn(move || {
        let _ = std::process::Command::new("powershell")
            .args(["-WindowStyle", "Hidden", "-NonInteractive", "-Command", &script])
            .spawn();
    });
}

/// Version silencieuse : log seulement si PowerShell indisponible.
pub fn notify_update(version: &str, size_mb: u64) {
    log::info!("Notification mise à jour v{} ({} MB)", version, size_mb);
    show_toast(
        "Dictum — Mise à jour disponible",
        &format!("Version {} disponible ({} MB). Cliquer sur l'icône tray.", version, size_mb),
    );
}

pub fn notify_pause(paused: bool) {
    if paused {
        show_toast("Dictum — Pause", "Dictée suspendue. Cliquer tray pour reprendre.");
    } else {
        show_toast("Dictum — Reprise", "Dictée active.");
    }
}

/// Notifie quand une longue transcription est terminée (> 50 mots).
pub fn notify_transcription_done(words: usize, secs: f32) {
    if words >= 50 {
        show_toast(
            "Dictum — Transcription terminée",
            &format!("{} mots transcrits en {:.1}s", words, secs),
        );
    }
}

pub fn notify_ready(hotkey: &str, model: &str) {
    show_toast(
        "Dictum — Prêt",
        &format!("Maintenir {} pour dicter. Modèle : {}", hotkey, model),
    );
}
