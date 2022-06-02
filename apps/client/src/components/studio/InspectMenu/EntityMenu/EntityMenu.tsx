import produce from "immer";
import { useAtomValue } from "jotai";
import React from "react";

import { findEntityById } from "@wired-xr/scene";

import { selectedAtom } from "../../../../helpers/studio/atoms";
import { useStudioStore } from "../../../../helpers/studio/store";
import AmbientLightMenu from "./AmbientLightMenu";
import BoxMenu from "./BoxMenu";
import CylinderMenu from "./CylinderMenu";
import ModelMenu from "./ModelMenu";
import PointLightMenu from "./PointLightMenu";
import SphereMenu from "./SphereMenu";
import SpotLightMenu from "./SpotLightMenu";
import TextMenu from "./TextMenu";

export default function EntityMenu() {
  const selected = useAtomValue(selectedAtom);

  function handleChange(key: string, value: any) {
    if (!selected) return;
    const scene = useStudioStore.getState().scene;

    const newScene = produce(scene, (draft) => {
      const found = findEntityById(draft.tree, selected.id);
      if (!found) return;

      // @ts-ignore
      found.props[key] = value;
    });

    useStudioStore.setState({ scene: newScene });
  }

  switch (selected?.type) {
    case "Box":
      return <BoxMenu selected={selected as any} handleChange={handleChange} />;
    case "Sphere":
      return (
        <SphereMenu selected={selected as any} handleChange={handleChange} />
      );
    case "Cylinder":
      return (
        <CylinderMenu selected={selected as any} handleChange={handleChange} />
      );
    case "Model":
      return (
        <ModelMenu selected={selected as any} handleChange={handleChange} />
      );
    case "PointLight":
      return (
        <PointLightMenu
          selected={selected as any}
          handleChange={handleChange}
        />
      );
    case "AmbientLight":
    case "DirectionalLight":
      return (
        <AmbientLightMenu
          selected={selected as any}
          handleChange={handleChange}
        />
      );
    case "SpotLight":
      return (
        <SpotLightMenu selected={selected as any} handleChange={handleChange} />
      );
    case "Text":
      return (
        <TextMenu selected={selected as any} handleChange={handleChange} />
      );
    default:
      return null;
  }
}
