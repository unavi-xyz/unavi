import { LatticeSchedules } from "lattice-engine/core";
import { gltfPlugin } from "lattice-engine/gltf";
import { orbitPlugin } from "lattice-engine/orbit";
import { physicsPlugin } from "lattice-engine/physics";
import { playerPlugin } from "lattice-engine/player";
import { defaultPlugin } from "lattice-engine/utils";
import { vrmPlugin } from "lattice-engine/vrm";
import { WorldBuilder } from "thyseus";

import { ClientSchedules } from "./constants";
import { connectToHost } from "./systems/connectToHost";
import { initApp } from "./systems/initApp";
import { joinWorld } from "./systems/joinWorld";
import { parseWorld } from "./systems/parseWorld";
import { publishLocation } from "./systems/publishLocation";
import { setSkybox } from "./systems/setSkybox";

export function clientPlugin(builder: WorldBuilder) {
  builder
    .addPlugin(defaultPlugin)
    .addPlugin(gltfPlugin)
    .addPlugin(orbitPlugin)
    .addPlugin(physicsPlugin)
    .addPlugin(playerPlugin)
    .addPlugin(vrmPlugin)
    .addSystemsToSchedule(LatticeSchedules.Startup, initApp)
    .addSystemsToSchedule(LatticeSchedules.PostFixedUpdate, publishLocation)
    .addSystemsToSchedule(ClientSchedules.JoinWorld, joinWorld)
    .addSystemsToSchedule(ClientSchedules.ConnectToHost, connectToHost)
    .addSystems(parseWorld, setSkybox);
}
