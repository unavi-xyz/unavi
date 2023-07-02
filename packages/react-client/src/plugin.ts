import { LatticeSchedules } from "lattice-engine/core";
import { gltfPlugin } from "lattice-engine/gltf";
import { orbitPlugin } from "lattice-engine/orbit";
import { physicsPlugin } from "lattice-engine/physics";
import { playerPlugin } from "lattice-engine/player";
import { defaultPlugin } from "lattice-engine/utils";
import { vrmPlugin } from "lattice-engine/vrm";
import { WorldBuilder } from "thyseus";

import { initApp } from "./systems/initApp";
import { loadWorldModels } from "./systems/loadWorldModels";
import { setSkybox } from "./systems/setSkybox";
import { setWorld } from "./systems/setWorld";

export function clientPlugin(builder: WorldBuilder) {
  builder
    .addPlugin(defaultPlugin)
    .addPlugin(gltfPlugin)
    .addPlugin(orbitPlugin)
    .addPlugin(physicsPlugin)
    .addPlugin(playerPlugin)
    .addPlugin(vrmPlugin)
    .addSystemsToSchedule(LatticeSchedules.Startup, initApp)
    .addSystems(loadWorldModels, setSkybox, setWorld);
}
