import produce from "immer";
import { useAtomValue } from "jotai";
import React from "react";

import { findEntityById } from "@wired-xr/scene";

import { selectedAtom } from "../../../helpers/studio/atoms";
import { useStudioStore } from "../../../helpers/studio/store";
import { capitalize } from "../../../helpers/utils/capitalize";
import NumberInput from "./NumberInput";

function round(value: number, digits = 3) {
  return Math.round(value * 10 ** digits) / 10 ** digits;
}

export default function Geometry() {
  const selected = useAtomValue(selectedAtom);
  const scene = useStudioStore((state) => state.scene);

  const meshModule = selected?.modules.find((item) => item.type === "Mesh");

  if (!meshModule) return null;

  function handleChange(key: string, value: string) {
    if (!selected) return;

    const newScene = produce(scene, (draft) => {
      const found = findEntityById(draft.tree, selected.id);

      found?.modules.forEach((module) => {
        if (module.type === "Mesh") {
          //@ts-ignore
          module.props[key] = Number(value);
        } else if (module.type === "Collider") {
          //@ts-ignore
          module.props[key] = Number(value);
        }
      });
    });

    useStudioStore.setState({ scene: newScene });
  }

  return (
    <div className="space-y-1">
      <div className="flex items-center space-x-2">
        {Object.entries(meshModule.props).map(([key, value], i) => {
          return (
            <div key={key} className="flex items-center space-x-2">
              <div>{capitalize(key)}</div>
              <NumberInput
                updatedValue={String(round(value))}
                onChange={(e) => handleChange(key, e.currentTarget.value)}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}
