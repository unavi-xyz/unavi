use crate::runtime::shared::Api;

pub struct EventReceptorRes;

#[derive(Default)]
pub struct WiredEventApi;

#[derive(Default)]
pub enum EventScope {
    #[default]
    Global,
    Spatial(f32),
}

#[derive(Default)]
pub struct EventFilter {
    pub node: Option<u32>,
    pub scope: EventScope,
    pub documents: Option<Vec<Vec<u8>>>,
}

pub enum EventSender {
    Global,
    Spatial,
}

pub struct Event {
    pub channel: String,
    pub payload: Vec<u8>,
    pub sender: EventSender,
    pub sender_document: Vec<u8>,
    pub time: u64,
}

pub fn emit(
    _api: &Api,
    _channel: String,
    _payload: Vec<u8>,
    _filter: EventFilter,
) -> anyhow::Result<()> {
    todo!()
}

pub fn listen(_api: &Api, _channels: Vec<String>, _filter: EventFilter) -> anyhow::Result<u32> {
    todo!()
}

pub fn receptor_poll(_api: &Api, _rep: u32) -> anyhow::Result<Option<Event>> {
    todo!()
}

pub fn receptor_drop(_api: &Api, _rep: u32) -> anyhow::Result<()> {
    todo!()
}
