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

impl RecordHandle {
    /// Start recording. The cpal stream lives in its own thread (Stream is !Send).
    pub fn start(device_name: Option<&str>) -> Result<Self> {
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
                    return Err(anyhow!("Microphone '{}' not found", name));
                }
            } else if host.default_input_device().is_none() {
                return Err(anyhow!("No default input device available"));
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

            // Block until stop signal
            let _ = stop_rx.recv();
            drop(stream);

            let recorded = samples.lock().unwrap().clone();
            let _ = samples_tx.send(recorded);
        });

        Ok(Self { stop_tx, samples_rx })
    }

    /// Stop recording and return captured samples (16 kHz mono f32).
    pub fn stop(self) -> Vec<f32> {
        let _ = self.stop_tx.send(());
        self.samples_rx.recv().unwrap_or_default()
    }
}

/// List available input device names.
pub fn list_devices() -> Vec<String> {
    let host = cpal::default_host();
    host.input_devices()
        .map(|devs| devs.filter_map(|d| d.name().ok()).collect())
        .unwrap_or_default()
}
