import { World as WorldSchema } from "@wired-protocol/types";
import { Asset, Warehouse } from "lattice-engine/core";
import { Gltf } from "lattice-engine/gltf";
import {
  Commands,
  dropStruct,
  Entity,
  Mut,
  Query,
  Res,
  Without,
  World,
} from "thyseus";

import { EngineSchedules } from "../../constants";
import { WorldJson } from "../components";

export async function parseWorld(
  world: World,
  commands: Commands,
  warehouse: Res<Warehouse>,
  worlds: Query<[Entity, Asset, Mut<WorldJson>], Without<Gltf>>
) {
  for (const [entity, asset, json] of worlds) {
    const buffer = asset.data.read(warehouse);
    if (!buffer || buffer.byteLength === 0) continue;

    const array = new Uint8Array(buffer);
    const parsed = WorldSchema.fromBinary(array);

    // Load model
    const gltf = new Gltf(parsed.model);
    commands.getById(entity.id).add(gltf);
    dropStruct(gltf);

    // Connect to host
    json.host = parsed.host ?? "";
    await world.runSchedule(EngineSchedules.ConnectToHost);
  }
}
