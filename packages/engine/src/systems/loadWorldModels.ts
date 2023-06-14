import { WorldMetadataSchema } from "@wired-protocol/types";
import { Asset, Warehouse } from "lattice-engine/core";
import { Gltf } from "lattice-engine/gltf";
import { dropStruct, Entity, Query, Res, With, Without } from "thyseus";

import { WorldJson } from "../components";

const decoder = new TextDecoder();

export function loadWorldModels(
  warehouse: Res<Warehouse>,
  worlds: Query<[Entity, Asset], [With<WorldJson>, Without<Gltf>]>
) {
  for (const [entity, asset] of worlds) {
    const buffer = asset.data.read(warehouse);
    if (!buffer || buffer.byteLength === 0) continue;

    const text = decoder.decode(buffer);
    const parsed = WorldMetadataSchema.safeParse(JSON.parse(text));

    if (!parsed.success) {
      console.warn(`Failed to parse world metadata ${asset.uri}:`, parsed.error);
      continue;
    }

    console.log("🤪", parsed.data.model);
    const gltf = new Gltf(parsed.data.model);
    entity.add(gltf);
    dropStruct(gltf);
  }
}
