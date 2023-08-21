import { Warehouse } from "lattice-engine/core";
import { ExportedGltf } from "lattice-engine/gltf";
import { EventReader, Res } from "thyseus";

import { useClientStore } from "../clientStore";

export function saveExport(
  warehouse: Res<Warehouse>,
  reader: EventReader<ExportedGltf>
) {
  if (reader.length === 0) return;

  useClientStore.setState({ exportedModel: null });

  for (const event of reader) {
    const uri = event.uri.read(warehouse);
    if (!uri) continue;

    fetch(uri)
      .then((response) => response.blob())
      .then((blob) => useClientStore.setState({ exportedModel: blob }));
  }

  reader.clear();
}
