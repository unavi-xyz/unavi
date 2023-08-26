import { atom, getDefaultStore } from "jotai";
import { Warehouse } from "lattice-engine/core";
import { Name, Scene, SceneStruct } from "lattice-engine/scene";
import { Entity, Query, Res } from "thyseus";

export const rootNameAtom = atom<string | null>(null);

export function setRootName(
  warehouse: Res<Warehouse>,
  sceneStruct: Res<SceneStruct>,
  scenes: Query<[Entity, Scene]>,
  names: Query<[Entity, Name]>
) {
  for (const [entity, scene] of scenes) {
    if (sceneStruct.activeScene !== entity.id) continue;

    for (const [entity, name] of names) {
      if (scene.rootId !== entity.id) continue;

      const rootName = name.value.read(warehouse);
      getDefaultStore().set(rootNameAtom, rootName || null);
    }
  }
}
