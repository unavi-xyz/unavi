use bevy::ecs::message::MessageWriter;
use schminput_rebinding::config::{LoadSchminputConfig, SaveSchminputConfig};


pub fn load_config(mut load: MessageWriter<LoadSchminputConfig>) {
    load.write_default();
}

pub fn save_config(mut save: MessageWriter<SaveSchminputConfig>) {
    save.write_default();
}
