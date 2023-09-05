import { atom, getDefaultStore } from "jotai";
import { ExportedGltf } from "lattice-engine/gltf";
import { EventReader } from "thyseus";

export const exportedModelAtom = atom<Blob | null>(null);

export function saveExport(reader: EventReader<ExportedGltf>) {
  if (reader.length === 0) return;

  getDefaultStore().set(exportedModelAtom, null);

  for (const event of reader) {
    const uri = event.uri;
    if (!uri) continue;

    fetch(uri)
      .then((response) => response.blob())
      .then((blob) => getDefaultStore().set(exportedModelAtom, blob));
  }

  reader.clear();
}
