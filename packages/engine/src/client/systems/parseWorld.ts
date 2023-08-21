import { World as WorldSchema } from "@wired-protocol/types";
import { Asset, Warehouse } from "lattice-engine/core";
import { Gltf } from "lattice-engine/gltf";
import { Commands, Entity, Mut, Query, Res, Without, World } from "thyseus";

import { EngineSchedules } from "../../constants";
import { WorldJson } from "../components";

const decoder = new TextDecoder();

export async function parseWorld(
  world: World,
  commands: Commands,
  warehouse: Res<Mut<Warehouse>>,
  worlds: Query<[Entity, Asset, Mut<WorldJson>], Without<Gltf>>
) {
  for (const [entity, asset, json] of worlds) {
    const buffer = asset.data.read(warehouse);
    if (!buffer || buffer.byteLength === 0) continue;

    const text = decoder.decode(buffer);
    const parsed = WorldSchema.fromJsonString(text);

    // Load model
    const gltf = new Gltf();
    gltf.uri.write(parsed.model, warehouse);
    commands.getById(entity.id).add(gltf);

    // Connect to host
    json.host.write(parsed.host ?? "", warehouse);
    await world.runSchedule(EngineSchedules.ConnectToHost);
  }
}
