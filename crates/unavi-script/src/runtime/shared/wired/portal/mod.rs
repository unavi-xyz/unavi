use crate::runtime::shared::Api;

pub struct PortalRes;

#[derive(Default)]
pub struct WiredPortalApi;

pub struct PortalDestination {
    pub space:  Vec<u8>,
    pub portal: Option<String>,
}

pub struct PortalTransform {
    pub translation: [f32; 3],
    pub rotation:    [f32; 4],
    pub scale:       [f32; 3],
}

impl Default for PortalTransform {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation:    [0.0, 0.0, 0.0, 1.0],
            scale:       [1.0; 3],
        }
    }
}

pub struct PortalParams {
    pub destination: PortalDestination,
    pub size:        [f32; 2],
    pub transform:   PortalTransform,
}

pub fn list_portals(_api: &Api) -> anyhow::Result<Vec<u32>> {
    todo!()
}

pub fn open_portal(_api: &Api, _params: PortalParams) -> anyhow::Result<u32> {
    todo!()
}

pub fn id(_api: &Api, _rep: u32) -> anyhow::Result<String> {
    todo!()
}

pub fn destination(_api: &Api, _rep: u32) -> anyhow::Result<PortalDestination> {
    todo!()
}

pub fn close(_api: &Api, _rep: u32) -> anyhow::Result<()> {
    todo!()
}

pub fn on_drop(_api: &Api, _rep: u32) -> anyhow::Result<()> {
    todo!()
}
