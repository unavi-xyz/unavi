import { ExportedGltf } from "lattice-engine/gltf";
import { EventReader } from "thyseus";

import { useClientStore } from "../clientStore";

export function saveExport(reader: EventReader<ExportedGltf>) {
  if (reader.length === 0) return;

  useClientStore.setState({ exportedModel: null });

  for (const event of reader) {
    fetch(event.uri)
      .then((response) => response.blob())
      .then((blob) => useClientStore.setState({ exportedModel: blob }));
  }

  reader.clear();
}
