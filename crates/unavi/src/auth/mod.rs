use bevy::prelude::*;

#[derive(Event, Default)]
pub struct LoginEvent;

pub fn trigger_login(mut writer: EventWriter<LoginEvent>) {
    writer.write_default();
}

pub fn handle_login(_: Trigger<LoginEvent>) {}
