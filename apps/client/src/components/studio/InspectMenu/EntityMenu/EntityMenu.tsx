import produce from "immer";
import { useAtomValue } from "jotai";
import React from "react";

import { findEntityById } from "@wired-xr/scene";

import { selectedAtom } from "../../../../helpers/studio/atoms";
import { useStudioStore } from "../../../../helpers/studio/store";
import BoxMenu from "./BoxMenu";
import SphereMenu from "./SphereMenu";

export default function EntityMenu() {
  const selected = useAtomValue(selectedAtom);
  const scene = useStudioStore((state) => state.scene);

  function handleChange(key: string, value: any) {
    if (!selected) return;

    const newScene = produce(scene, (draft) => {
      const found = findEntityById(draft.tree, selected.id);
      if (!found) return;

      // @ts-ignore
      found.props[key] = value;
    });

    useStudioStore.setState({ scene: newScene });
  }

  if (!selected) return null;

  if (selected.type === "Box") {
    return <BoxMenu selected={selected as any} handleChange={handleChange} />;
  }

  if (selected.type === "Sphere")
    return (
      <SphereMenu selected={selected as any} handleChange={handleChange} />
    );

  return null;
}
