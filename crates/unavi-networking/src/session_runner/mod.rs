use bevy::{prelude::*, utils::tracing::Instrument};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::LocalSet,
};

mod connect;
pub mod handler;

#[derive(Resource)]
pub struct SessionRunner {
    pub sender: UnboundedSender<NewSession>,
}

pub struct NewSession {
    pub address: String,
    pub receiver: UnboundedReceiver<InstanceAction>,
    pub record_id: String,
}

#[derive(Debug)]
pub enum InstanceAction {
    Close,
    SendDatagram(Box<[u8]>),
}

impl Default for SessionRunner {
    fn default() -> Self {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<NewSession>();

        let task = async move {
            while let Some(new_session) = receiver.recv().await {
                let span = info_span!("Session", address = new_session.address);

                tokio::task::spawn_local(
                    async move {
                        match handler::handle_session(new_session).await {
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
