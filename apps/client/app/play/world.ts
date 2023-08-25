import { enginePlugin } from "@unavi/engine";
import { Engine } from "lattice-engine/core";
import { World } from "thyseus";

export let world: World | undefined;

export async function resetWorld() {
  world = await Engine.createWorld().addPlugin(enginePlugin).build();
  return world;
}
