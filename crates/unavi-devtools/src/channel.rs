use async_channel::{
    Receiver,
    Sender,
};
use bevy::prelude::*;

/// A typed bridge from off-world producers into the ECS.
///
/// Each crate registers its own message type, keeping the bus decentralized
/// rather than funneling every variant through one shared enum.
#[derive(Resource)]
pub struct DevChannel<T: Message> {
    tx: Sender<T>,
    rx: Receiver<T>,
}

impl<T: Message> DevChannel<T> {
    /// A sender clone for use outside the ECS.
    #[must_use]
    pub fn sender(&self) -> Sender<T> {
        self.tx.clone()
    }
}

pub trait DevChannelAppExt {
    /// Registers a [`DevChannel<T>`] resource and forwards its messages into
    /// `MessageReader<T>` each frame.
    fn add_dev_channel<T: Message>(&mut self) -> &mut Self;
}

impl DevChannelAppExt for App {
    fn add_dev_channel<T: Message>(&mut self) -> &mut Self {
        let (tx, rx) = async_channel::unbounded::<T>();
        self.insert_resource(DevChannel { tx, rx })
            .add_message::<T>()
            .add_systems(PreUpdate, drain::<T>)
    }
}

fn drain<T: Message>(channel: Res<DevChannel<T>>, mut writer: MessageWriter<T>) {
    while let Ok(msg) = channel.rx.try_recv() {
        writer.write(msg);
    }
}
