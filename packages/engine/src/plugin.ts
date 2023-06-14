import { LatticeSchedules } from "lattice-engine/core";
import { gltfPlugin } from "lattice-engine/gltf";
import { orbitPlugin } from "lattice-engine/orbit";
import { defaultPlugin } from "lattice-engine/utils";
import { WorldBuilder } from "thyseus";

import { initApp } from "./systems/initApp";
import { loadWorldModels } from "./systems/loadWorldModels";

export function appPlugin(builder: WorldBuilder) {
  builder
    .addPlugin(defaultPlugin)
    .addPlugin(orbitPlugin)
    .addPlugin(gltfPlugin)
    .addSystemsToSchedule(LatticeSchedules.Startup, initApp)
    .addSystems(loadWorldModels);
}
