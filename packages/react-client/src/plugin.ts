import { LatticeSchedules } from "lattice-engine/core";
import { gltfPlugin } from "lattice-engine/gltf";
import { orbitPlugin } from "lattice-engine/orbit";
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
    .addPlugin(orbitPlugin)
    .addPlugin(playerPlugin)
    .addPlugin(vrmPlugin)
    .addPlugin(gltfPlugin)
    .addSystemsToSchedule(LatticeSchedules.Startup, initApp)
    .addSystems(loadWorldModels, setSkybox, setWorld);
}
