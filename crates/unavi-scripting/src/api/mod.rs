use wired::{dwn::WiredDwn, log::WiredLog, player::WiredPlayer, scene::WiredScene};

pub(crate) mod utils;
pub mod wired;

#[derive(Default)]
pub struct ApiData {
    pub wired_dwn: Option<WiredDwn>,
    pub wired_log: Option<WiredLog>,
    pub wired_player: Option<WiredPlayer>,
    pub wired_scene: Option<WiredScene>,
}
