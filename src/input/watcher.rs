use std::any::TypeId;
use std::path::PathBuf;

use cosmic::iced::{
    Subscription,
    futures::{SinkExt, channel::mpsc},
    stream,
};
use evdev::{Device, EventType, KeyCode};
use tracing::{debug, info, trace, warn};

use super::classify::{DeviceKind, Side, classify, key_side};

#[derive(Clone, Copy, Debug)]
pub enum InputEvent {
    Down(Option<Side>),
    Up(Option<Side>),
}

pub fn subscription() -> Subscription<InputEvent> {
    Subscription::run_with(TypeId::of::<InputEvent>(), |_| {
        stream::channel(64, move |mut output| async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel::<InputEvent>(64);

            for path in enumerate_event_paths() {
                let dev = match Device::open(&path) {
                    Ok(d) => d,
                    Err(e) => {
                        debug!(?path, %e, "cannot open device");
                        continue;
                    }
                };
                let Some(kind) = classify(&dev) else { continue };
                let name = dev.name().unwrap_or("?").to_string();
                info!(?path, %name, ?kind, "watching device");

                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = read_device(dev, kind, tx).await {
                        warn!(?path, %e, "device reader stopped");
                    }
                });
            }
            drop(tx);

            forward(&mut output, &mut rx).await;
        })
    })
}

fn enumerate_event_paths() -> Vec<PathBuf> {
    let Ok(dir) = std::fs::read_dir("/dev/input") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in dir.flatten() {
        let path = entry.path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("event"))
        {
            out.push(path);
        }
    }
    out.sort();
    out
}

async fn read_device(
    dev: Device,
    kind: DeviceKind,
    tx: tokio::sync::mpsc::Sender<InputEvent>,
) -> std::io::Result<()> {
    let mut stream = dev.into_event_stream()?;
    loop {
        let event = stream.next_event().await?;
        if event.event_type() != EventType::KEY {
            continue;
        }
        let side = key_side(KeyCode::new(event.code()), kind);
        let emit = match event.value() {
            1 => Some(InputEvent::Down(side)),
            0 => Some(InputEvent::Up(side)),
            _ => None,
        };
        if let Some(ev) = emit {
            trace!(?ev, ?kind, "input");
            if tx.send(ev).await.is_err() {
                return Ok(());
            }
        }
    }
}

async fn forward(
    output: &mut mpsc::Sender<InputEvent>,
    rx: &mut tokio::sync::mpsc::Receiver<InputEvent>,
) {
    while let Some(ev) = rx.recv().await {
        if output.send(ev).await.is_err() {
            return;
        }
    }
}
