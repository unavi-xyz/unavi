import { LatticeSchedules } from "lattice-engine/core";
import { gltfPlugin } from "lattice-engine/gltf";
import { orbitPlugin } from "lattice-engine/orbit";
import { physicsPlugin } from "lattice-engine/physics";
import { playerPlugin } from "lattice-engine/player";
import { defaultPlugin } from "lattice-engine/utils";
import { vrmPlugin } from "lattice-engine/vrm";
import { run, WorldBuilder } from "thyseus";

import { ClientSchedules } from "./constants";
import { connectToHost } from "./systems/connectToHost";
import { initApp } from "./systems/initApp";
import { joinWorld } from "./systems/joinWorld";
import { lerpTransforms } from "./systems/lerpTransforms";
import { movePlayers } from "./systems/movePlayers";
import { parseWorld } from "./systems/parseWorld";
import { publishLocation } from "./systems/publishLocation";
import { sendEvents } from "./systems/sendEvents";
import { setLocationUpdateTime } from "./systems/setLocationUpdateTime";
import { setSkybox } from "./systems/setSkybox";
import { spawnPlayers } from "./systems/spawnPlayers";

export function clientPlugin(builder: WorldBuilder) {
  builder
    .addPlugin(defaultPlugin)
    .addPlugin(gltfPlugin)
    .addPlugin(orbitPlugin)
    .addPlugin(physicsPlugin)
    .addPlugin(playerPlugin)
    .addPlugin(vrmPlugin)
    .addSystemsToSchedule(LatticeSchedules.Startup, initApp)
    .addSystemsToSchedule(LatticeSchedules.PreUpdate, sendEvents)
    .addSystemsToSchedule(LatticeSchedules.PostFixedUpdate, publishLocation)
    .addSystemsToSchedule(ClientSchedules.ConnectToHost, connectToHost)
    .addSystems(
      joinWorld,
      movePlayers,
      parseWorld,
      setSkybox,
      spawnPlayers,
      ...run.chain(setLocationUpdateTime, lerpTransforms, movePlayers)
    );
}
