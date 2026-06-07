use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, StreamConfig};
use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::{anyhow, Result};
use crossbeam_channel::{bounded, Receiver, Sender};

/// Sends stop signal, receives recorded samples back.
pub struct RecordHandle {
    stop_tx: Sender<()>,
    samples_rx: Receiver<Vec<f32>>,
}

/// Beep court via Windows API (non bloquant, spawné dans un thread).
pub fn beep(freq_hz: u32, duration_ms: u32) {
    #[cfg(windows)]
    std::thread::spawn(move || unsafe {
        windows_beep(freq_hz, duration_ms);
    });
}

#[cfg(windows)]
unsafe fn windows_beep(freq: u32, dur: u32) {
    #[link(name = "kernel32")]
    extern "system" {
        fn Beep(dwFreq: u32, dwDuration: u32) -> i32;
    }
    Beep(freq, dur);
}

impl RecordHandle {
    /// Start recording. The cpal stream lives in its own thread (Stream is !Send).
    pub fn start(device_name: Option<&str>, max_secs: u64) -> Result<Self> {
        log::info!("Démarrage enregistrement (max {}s)", max_secs);
        let device_name = device_name.map(String::from);
        let (stop_tx, stop_rx) = bounded::<()>(1);
        let (samples_tx, samples_rx) = bounded::<Vec<f32>>(1);

        // Validate device exists before spawning thread
        {
            let host = cpal::default_host();
            if let Some(name) = &device_name {
                let found = host
                    .input_devices()?
                    .any(|d| d.name().unwrap_or_default() == *name);
                if !found {
                    let available = host.input_devices()
                        .map(|d| d.filter_map(|d| d.name().ok()).collect::<Vec<_>>().join(", "))
                        .unwrap_or_default();
                    return Err(anyhow!(
                        "Microphone '{}' introuvable.\nDisponibles : {}",
                        name, if available.is_empty() { "aucun".to_string() } else { available }
                    ));
                }
            } else if host.default_input_device().is_none() {
                return Err(anyhow!("Aucun microphone détecté. Brancher un micro et réessayer."));
            }
        }

        thread::spawn(move || {
            let host = cpal::default_host();
            let device = if let Some(name) = &device_name {
                host.input_devices()
                    .unwrap()
                    .find(|d| d.name().unwrap_or_default() == *name)
                    .unwrap()
            } else {
                host.default_input_device().unwrap()
            };
            log::info!("Micro actif : {}", device.name().unwrap_or_else(|_| "inconnu".to_string()));

            let config = StreamConfig {
                channels: 1,
                sample_rate: SampleRate(16000),
                buffer_size: cpal::BufferSize::Default,
            };

            let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
            let samples_write = samples.clone();

            let stream = device
                .build_input_stream(
                    &config,
                    move |data: &[f32], _| {
                        if let Ok(mut buf) = samples_write.lock() {
                            buf.extend_from_slice(data);
                        }
                    },
                    |err| log::error!("Audio stream error: {err}"),
                    None,
                )
                .expect("Failed to build input stream");

            stream.play().expect("Failed to start audio stream");

            // Beep démarrage géré depuis main.rs (config.beep_enabled)

            // Block jusqu'au signal stop ou timeout max_record
            let timeout = std::time::Duration::from_secs(max_secs);
            let timed_out = stop_rx.recv_timeout(timeout).is_err();
            if timed_out {
                log::warn!("Enregistrement stoppé par timeout ({}s)", max_secs);
            }
            drop(stream);

            let recorded = samples.lock().unwrap().clone();

            // Beep fin seulement si enregistrement assez long (> 0.3s = 4800 samples)
            if recorded.len() > 4800 {
                #[cfg(windows)]
                unsafe {
                    if timed_out {
                        // Beep double si timeout
                        windows_beep(800, 60);
                        windows_beep(600, 60);
                    } else {
                        windows_beep(600, 80);
                    }
                }
            }
            let _ = samples_tx.send(recorded);
        });

        Ok(Self { stop_tx, samples_rx })
    }

    /// Stop recording and return captured samples (16 kHz mono f32).
    pub fn stop(self) -> Vec<f32> {
        let _ = self.stop_tx.send(());
        let samples = self.samples_rx.recv().unwrap_or_default();
        let duration_secs = samples.len() as f32 / 16000.0;
        let size_kb = samples.len() * 4 / 1024;
        log::info!("Enregistrement arrêté : {:.1}s ({} samples, {} KB f32)", duration_secs, samples.len(), size_kb);
        samples
    }
}

/// Liste les microphones disponibles. Le défaut système est marqué avec *.
pub fn list_devices() -> Vec<String> {
    let host = cpal::default_host();
    let default_name = host.default_input_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    host.input_devices()
        .map(|devs| devs.filter_map(|d| {
            d.name().ok().map(|name| {
                if name == default_name {
                    format!("{} (défaut)", name)
                } else {
                    name
                }
            })
        }).collect())
        .unwrap_or_default()
}
