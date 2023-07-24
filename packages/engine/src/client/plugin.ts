import { LatticeSchedules } from "lattice-engine/core";
import { csmPlugin } from "lattice-engine/csm";
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

import { EngineSchedules } from "../constants";
import { calcPlayerVelocity } from "./systems/calcPlayerVelocity";
import { connectToHost } from "./systems/connectToHost";
import { exportLoadingInfo } from "./systems/exportLoadingInfo";
import { initApp } from "./systems/initApp";
import { joinWorld } from "./systems/joinWorld";
import { lerpTransforms } from "./systems/lerpTransforms";
import { movePlayers } from "./systems/movePlayers";
import { parseWorld } from "./systems/parseWorld";
import { publishLocation } from "./systems/publishLocation";
import { saveExport } from "./systems/saveExport";
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
    .addPlugin(csmPlugin)
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
    .addSystemsToSchedule(EngineSchedules.ConnectToHost, connectToHost)
    .addSystems(
      exportLoadingInfo,
      joinWorld,
      movePlayers,
      parseWorld,
      saveExport,
      setPlayersAirTime,
      setPlayersAvatar,
      setRootName,
      setSkybox,
      setUserAvatar,
      spawnPlayers,
      ...run.chain(setLocationUpdateTime, lerpTransforms, [
        calcPlayerVelocity,
        movePlayers,
      ])
    );
}
