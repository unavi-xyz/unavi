import { atom, getDefaultStore } from "jotai";
import { Warehouse } from "lattice-engine/core";
import { ExportedGltf } from "lattice-engine/gltf";
import { EventReader, Res } from "thyseus";

export const exportedModelAtom = atom<Blob | null>(null);

export function saveExport(
  warehouse: Res<Warehouse>,
  reader: EventReader<ExportedGltf>
) {
  if (reader.length === 0) return;

  getDefaultStore().set(exportedModelAtom, null);

  for (const event of reader) {
    const uri = event.uri.read(warehouse);
    if (!uri) continue;

    fetch(uri)
      .then((response) => response.blob())
      .then((blob) => getDefaultStore().set(exportedModelAtom, blob));
  }

  reader.clear();
}
