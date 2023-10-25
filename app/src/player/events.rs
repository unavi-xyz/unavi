use bevy::prelude::*;
use std::ops::Deref;

#[derive(Event, Debug, Default)]
pub struct LookDeltaEvent {
    rotation_delta: Vec3,
}

impl LookDeltaEvent {
    pub fn new(other: &Vec3) -> Self {
        Self {
            rotation_delta: *other,
        }
    }
}

impl Deref for LookDeltaEvent {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.rotation_delta
    }
}

#[derive(Event, Debug, Default)]
pub struct LookEvent {
    rotation: Vec3,
}

impl LookEvent {
    pub fn new(other: &Vec3) -> Self {
        Self { rotation: *other }
    }
}

impl Deref for LookEvent {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.rotation
    }
}

#[derive(Event, Debug, Default)]
pub struct PitchEvent {
    pitch: f32,
}

impl PitchEvent {
    pub fn new(value: f32) -> Self {
        Self { pitch: value }
    }
}

impl Deref for PitchEvent {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.pitch
    }
}

#[derive(Event, Debug, Default)]
pub struct YawEvent {
    yaw: f32,
}

impl YawEvent {
    pub fn new(value: f32) -> Self {
        Self { yaw: value }
    }
}

impl Deref for YawEvent {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.yaw
    }
}
