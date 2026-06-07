/// API HTTP locale Dictum — écoute sur 127.0.0.1:44880

use std::sync::Arc;
use crate::AppState;
use crossbeam_channel::Sender;
use crate::AppEvent;

pub const API_PORT: u16 = 44880;

fn respond_json(request: tiny_http::Request, body: &str) {
    let resp = tiny_http::Response::from_string(body)
        .with_header(tiny_http::Header::from_bytes(b"Content-Type", b"application/json; charset=utf-8").unwrap())
        .with_header(tiny_http::Header::from_bytes(b"Access-Control-Allow-Origin", b"*").unwrap());
    let _ = request.respond(resp);
}

fn respond_text(request: tiny_http::Request, body: &str, status: u16) {
    let resp = tiny_http::Response::from_string(body)
        .with_status_code(status)
        .with_header(tiny_http::Header::from_bytes(b"Content-Type", b"text/plain; charset=utf-8").unwrap())
        .with_header(tiny_http::Header::from_bytes(b"Access-Control-Allow-Origin", b"*").unwrap());
    let _ = request.respond(resp);
}

pub fn start(state: AppState, event_tx: Sender<AppEvent>) {
    let addr = format!("127.0.0.1:{}", API_PORT);
    let server = match tiny_http::Server::http(&addr) {
        Ok(s) => {
            log::info!("API HTTP locale démarrée sur http://{}", addr);
            Arc::new(s)
        }
        Err(e) => {
            log::warn!("API HTTP : impossible de démarrer sur {} : {}", addr, e);
            return;
        }
    };

    std::thread::spawn(move || {
        for mut request in server.incoming_requests() {
            let method = request.method().to_string();
            let url = request.url().to_string();
            let path = url.split('?').next().unwrap_or(&url).to_string();
            log::debug!("API {} {}", method, path);

            match (method.as_str(), path.as_str()) {

                // GET /status
                ("GET", "/status") => {
                    let recording = *state.is_recording.lock().unwrap();
                    let transcribing = *state.is_transcribing.lock().unwrap();
                    let paused = *state.is_paused.lock().unwrap();
                    let count = *state.session_count.lock().unwrap();
                    let cfg = state.config.lock().unwrap();
                    let body = format!(
                        r#"{{"recording":{},"transcribing":{},"paused":{},"session_count":{},"hotkey":"{}","model":"{}","language":"{}","version":"{}"}}"#,
                        recording, transcribing, paused, count,
                        cfg.hotkey_string(), cfg.model_name(), cfg.language,
                        env!("CARGO_PKG_VERSION")
                    );
                    respond_json(request, &body);
                }

                // GET /history
                ("GET", "/history") => {
                    let hist = state.history.lock().unwrap();
                    let entries: Vec<String> = hist.entries().iter().map(|e| {
                        format!(r#"{{"text":{},"timestamp":{}}}"#,
                            serde_json::to_string(&e.text).unwrap_or_else(|_| "\"\"".into()),
                            e.timestamp)
                    }).collect();
                    let body = format!("[{}]", entries.join(","));
                    respond_json(request, &body);
                }

                // GET /config
                ("GET", "/config") => {
                    let cfg = state.config.lock().unwrap();
                    let body = serde_json::to_string(&*cfg).unwrap_or_else(|_| "{}".into());
                    respond_json(request, &body);
                }

                // POST /inject
                ("POST", "/inject") => {
                    let mut body = String::new();
                    request.as_reader().read_to_string(&mut body).ok();
                    let text = body.trim().to_string();
                    if text.is_empty() {
                        respond_text(request, "error: empty body", 400);
                    } else {
                        let cfg = state.config.lock().unwrap().clone();
                        std::thread::spawn(move || crate::inject::inject_text(&text, &cfg));
                        respond_json(request, r#"{"ok":true}"#);
                    }
                }

                // POST /pause
                ("POST", "/pause") => {
                    let mut p = state.is_paused.lock().unwrap();
                    *p = !*p;
                    let new_state = *p;
                    drop(p);
                    respond_json(request, &format!(r#"{{"paused":{}}}"#, new_state));
                }

                // POST /reload
                ("POST", "/reload") => {
                    let _ = event_tx.send(AppEvent::ReloadConfig);
                    respond_json(request, r#"{"ok":true}"#);
                }

                // POST /reformulate
                ("POST", "/reformulate") => {
                    let mut body = String::new();
                    request.as_reader().read_to_string(&mut body).ok();
                    let parsed: Option<serde_json::Value> = serde_json::from_str(&body).ok();
                    let text = parsed.as_ref().and_then(|v| v["text"].as_str()).unwrap_or("").to_string();
                    let style = parsed.as_ref().and_then(|v| v["style"].as_str()).unwrap_or("formel").to_string();
                    if text.is_empty() {
                        respond_text(request, "error: missing text", 400);
                    } else {
                        let cfg = state.config.lock().unwrap().clone();
                        match crate::reformulate::reformulate(&text, &style, &cfg.ollama_model, &cfg.ollama_url) {
                            Ok(result) => {
                                let body = format!(r#"{{"result":{}}}"#,
                                    serde_json::to_string(&result).unwrap_or_else(|_| "\"\"".into()));
                                respond_json(request, &body);
                            }
                            Err(e) => {
                                let body = format!(r#"{{"error":{}}}"#,
                                    serde_json::to_string(&e.to_string()).unwrap_or_else(|_| "\"\"".into()));
                                respond_json(request, &body);
                            }
                        }
                    }
                }

                // OPTIONS (CORS)
                ("OPTIONS", _) => {
                    let resp = tiny_http::Response::empty(200)
                        .with_header(tiny_http::Header::from_bytes(b"Access-Control-Allow-Origin", b"*").unwrap())
                        .with_header(tiny_http::Header::from_bytes(b"Access-Control-Allow-Methods", b"GET, POST, OPTIONS").unwrap());
                    let _ = request.respond(resp);
                }

                // 404
                _ => respond_text(request, "not found", 404),
            }
        }
    });
}
