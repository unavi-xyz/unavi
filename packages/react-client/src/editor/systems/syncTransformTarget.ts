import { TransformControls } from "lattice-engine/transform";
import { Mut, Query } from "thyseus";

import { useSceneStore } from "../sceneStore";

let lastTransformTarget: bigint | undefined;

export function syncTransformTarget(
  transformControls: Query<Mut<TransformControls>>
) {
  for (const transformControl of transformControls) {
    const targetId = transformControl.targetId;

    if (targetId !== lastTransformTarget) {
      // Set UI from transform controls
      useSceneStore.setState({ selectedId: targetId });
    } else {
      // Set transform controls from UI
      transformControl.targetId = useSceneStore.getState().selectedId ?? 0n;
    }

    lastTransformTarget = targetId;
  }
}
