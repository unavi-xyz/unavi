import { corePlugin, LatticeSchedules } from "lattice-engine/core";
import { gltfPlugin } from "lattice-engine/gltf";
import { inputPlugin } from "lattice-engine/input";
import { orbitPlugin } from "lattice-engine/orbit";
import { renderPlugin } from "lattice-engine/render";
import { scenePlugin } from "lattice-engine/scene";
import { WorldBuilder } from "thyseus";

import { initApp } from "./systems/initApp";
import { loadWorldModels } from "./systems/loadWorldModels";
import { setSkybox } from "./systems/setSkybox";
import { setWorld } from "./systems/setWorld";

export function clientPlugin(builder: WorldBuilder) {
  builder
    .addPlugin(corePlugin)
    .addPlugin(renderPlugin)
    .addPlugin(scenePlugin)
    .addPlugin(inputPlugin)
    .addPlugin(orbitPlugin)
    .addPlugin(gltfPlugin)
    .addSystemsToSchedule(LatticeSchedules.Startup, initApp)
    .addSystems(loadWorldModels, setSkybox, setWorld);
}
