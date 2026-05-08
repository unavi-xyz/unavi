use crate::runtime::shared::{Api, slot_map::SlotMap};

pub mod bridge;

pub struct AgentRes {}

#[derive(Default)]
pub struct WiredAgentApi {
    pub agents: SlotMap<AgentRes>,
}

#[derive(Clone, Copy, Debug)]
pub enum BoneName {
    Hips,
    Spine,
    Chest,
    UpperChest,
    Neck,
    Head,
    LeftEye,
    RightEye,
    Jaw,
    LeftShoulder,
    LeftUpperArm,
    LeftLowerArm,
    LeftHand,
    RightShoulder,
    RightUpperArm,
    RightLowerArm,
    RightHand,
    LeftUpperLeg,
    LeftLowerLeg,
    LeftFoot,
    LeftToes,
    RightUpperLeg,
    RightLowerLeg,
    RightFoot,
    RightToes,
    LeftThumbProximal,
    LeftThumbIntermediate,
    LeftThumbDistal,
    LeftIndexProximal,
    LeftIndexIntermediate,
    LeftIndexDistal,
    LeftMiddleProximal,
    LeftMiddleIntermediate,
    LeftMiddleDistal,
    LeftRingProximal,
    LeftRingIntermediate,
    LeftRingDistal,
    LeftLittleProximal,
    LeftLittleIntermediate,
    LeftLittleDistal,
    RightThumbProximal,
    RightThumbIntermediate,
    RightThumbDistal,
    RightIndexProximal,
    RightIndexIntermediate,
    RightIndexDistal,
    RightMiddleProximal,
    RightMiddleIntermediate,
    RightMiddleDistal,
    RightRingProximal,
    RightRingIntermediate,
    RightRingDistal,
    RightLittleProximal,
    RightLittleIntermediate,
    RightLittleDistal,
}

pub fn local_agent(api: &Api) -> anyhow::Result<u32> {
    Ok(api.wired_agent.try_lock()?.agents.insert(AgentRes {}))
}

pub fn local_camera(api: &Api) -> anyhow::Result<u32> {
    todo!()
}

pub fn bone(api: &Api, rep: u32, name: BoneName) -> anyhow::Result<Option<u32>> {
    let res = api
        .wired_agent
        .try_lock()?
        .agents
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("resource not found"))?;
    todo!()
}

pub fn on_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_agent.try_lock()?.agents.remove(rep);
    Ok(())
}
