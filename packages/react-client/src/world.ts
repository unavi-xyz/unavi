import { Engine } from "lattice-engine/core";
import { World } from "thyseus";

import { clientPlugin } from "./plugin";

export let world: World | undefined;

export async function resetWorld() {
  world = await Engine.createWorld().addPlugin(clientPlugin).build();
  return world;
}
