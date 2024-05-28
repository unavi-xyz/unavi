use bevy::{prelude::*, utils::tracing::Instrument};
use capnp::message::HeapAllocator;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::LocalSet,
};

use self::handler::handle_session;

mod connect;
mod handler;
mod rpc;

#[derive(Resource)]
pub struct NetworkingThread {
    pub sender: UnboundedSender<NewSession>,
}

pub struct NewSession {
    pub address: String,
    pub receiver: UnboundedReceiver<SessionRequest>,
    pub record_id: String,
    pub sender: UnboundedSender<SessionResponse>,
}

pub enum SessionRequest {
    Close,
    SendDatagram(capnp::message::Builder<HeapAllocator>),
}

pub enum SessionResponse {
    Tickrate(f32),
    PlayerTransform {
        player: u16,
        rotation: [f32; 4],
        translation: [f32; 3],
    },
}

impl Default for NetworkingThread {
    fn default() -> Self {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<NewSession>();

        let task = async move {
            while let Some(new_session) = receiver.recv().await {
                let span = info_span!("Session", address = new_session.address);

                tokio::task::spawn_local(
                    async move {
                        match handle_session(new_session).await {
                            Ok(_) => info!("Graceful exit."),
                            Err(e) => error!("{}", e),
                        };
                    }
                    .instrument(span),
                );
            }
        };

        #[cfg(not(target_family = "wasm"))]
        std::thread::spawn(move || {
            let local = LocalSet::new();
            local.spawn_local(task);
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(local);
        });

        #[cfg(target_family = "wasm")]
        let _ = wasm_bindgen_futures::future_to_promise(async move {
            let local = LocalSet::new();
            local.run_until(task).await;
            Ok(wasm_bindgen::JsValue::UNDEFINED)
        });

        Self { sender }
    }
}
