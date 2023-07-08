import { LatticeSchedules } from "lattice-engine/core";
import { gltfPlugin } from "lattice-engine/gltf";
import { orbitPlugin } from "lattice-engine/orbit";
import { physicsPlugin } from "lattice-engine/physics";
import { playerPlugin } from "lattice-engine/player";
import { postprocessingPlugin } from "lattice-engine/postprocessing";
import { textPlugin } from "lattice-engine/text";
import { transformPlugin } from "lattice-engine/transform";
import { defaultPlugin } from "lattice-engine/utils";
import { vrmPlugin } from "lattice-engine/vrm";
import { run, WorldBuilder } from "thyseus";

import { ClientSchedules } from "./constants";
import { calcPlayerVelocity } from "./systems/calcPlayerVelocity";
import { connectToHost } from "./systems/connectToHost";
import { addMeshes } from "./systems/editor/addMeshes";
import { addNodes } from "./systems/editor/addNodes";
import { enterEditMode } from "./systems/editor/enterEditMode";
import { exitEditMode } from "./systems/editor/exitEditMode";
import { exportLoadingInfo } from "./systems/exportLoadingInfo";
import { initApp } from "./systems/initApp";
import { joinWorld } from "./systems/joinWorld";
import { lerpTransforms } from "./systems/lerpTransforms";
import { movePlayers } from "./systems/movePlayers";
import { parseWorld } from "./systems/parseWorld";
import { publishLocation } from "./systems/publishLocation";
import { sendEvents } from "./systems/sendEvents";
import { setLocationUpdateTime } from "./systems/setLocationUpdateTime";
import { setPlayersAirTime } from "./systems/setPlayersAirTime";
import { setPlayersAvatar } from "./systems/setPlayersAvatars";
import { setRootName } from "./systems/setRootName";
import { setSkybox } from "./systems/setSkybox";
import { setUserAvatar } from "./systems/setUserAvatar";
import { spawnPlayers } from "./systems/spawnPlayers";

export function clientPlugin(builder: WorldBuilder) {
  builder
    .addPlugin(defaultPlugin)
    .addPlugin(gltfPlugin)
    .addPlugin(orbitPlugin)
    .addPlugin(physicsPlugin)
    .addPlugin(playerPlugin)
    .addPlugin(postprocessingPlugin)
    .addPlugin(textPlugin)
    .addPlugin(transformPlugin)
    .addPlugin(vrmPlugin)
    .addSystemsToSchedule(LatticeSchedules.Startup, initApp)
    .addSystemsToSchedule(LatticeSchedules.PreUpdate, sendEvents)
    .addSystemsToSchedule(LatticeSchedules.PostFixedUpdate, publishLocation)
    .addSystemsToSchedule(ClientSchedules.ConnectToHost, connectToHost)
    .addSystemsToSchedule(ClientSchedules.EnterEditMode, enterEditMode)
    .addSystemsToSchedule(ClientSchedules.ExitEditMode, exitEditMode)
    .addSystems(
      exportLoadingInfo,
      joinWorld,
      movePlayers,
      parseWorld,
      setPlayersAirTime,
      setPlayersAvatar,
      setRootName,
      setSkybox,
      setUserAvatar,
      spawnPlayers,
      addMeshes,
      addNodes,
      ...run.chain(setLocationUpdateTime, lerpTransforms, [
        calcPlayerVelocity,
        movePlayers,
      ])
    );
}
