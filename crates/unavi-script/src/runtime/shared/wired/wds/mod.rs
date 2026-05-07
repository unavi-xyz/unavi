use crate::runtime::shared::Api;

pub struct WdsRes;
pub struct QueryFutureRes;
pub struct ReadFutureRes;

#[derive(Default)]
pub struct WiredWdsApi;

pub struct QueryFilter {
    pub creator: Option<String>,
    pub schemas: Option<Vec<Vec<u8>>>,
}

pub struct WdsRecord {
    pub id: Vec<u8>,
    pub creator: String,
    pub schemas: Vec<Vec<u8>>,
    pub containers: Vec<(String, Vec<u8>)>,
}

pub fn get_wds(_api: &Api) -> anyhow::Result<u32> {
    todo!()
}

pub fn query(_api: &Api, _wds_rep: u32, _filter: Option<QueryFilter>) -> anyhow::Result<u32> {
    todo!()
}

pub fn read(_api: &Api, _wds_rep: u32, _record_id: Vec<u8>) -> anyhow::Result<u32> {
    todo!()
}

pub fn query_future_poll(
    _api: &Api,
    _rep: u32,
) -> anyhow::Result<Option<Result<Vec<Vec<u8>>, ()>>> {
    todo!()
}

pub fn read_future_poll(_api: &Api, _rep: u32) -> anyhow::Result<Option<Result<WdsRecord, ()>>> {
    todo!()
}

pub fn query_future_drop(_api: &Api, _rep: u32) -> anyhow::Result<()> {
    todo!()
}

pub fn read_future_drop(_api: &Api, _rep: u32) -> anyhow::Result<()> {
    todo!()
}

pub fn wds_drop(_api: &Api, _rep: u32) -> anyhow::Result<()> {
    todo!()
}
