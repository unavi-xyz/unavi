import { atom, getDefaultStore } from "jotai";
import { Engine } from "lattice-engine/core";
import { useEffect } from "react";

import Canvas from "./Canvas";
import { useWorld } from "./useWorld";

export const engineAtom = atom<Engine | null>(null);

interface Props {
  defaultAvatar?: string;
  skybox?: string;
  uri?: string;
}

export default function Client({ skybox, defaultAvatar, uri }: Props) {
  const world = useWorld();

  useEffect(() => {
    if (!world) return;

    const engine = new Engine(world);
    getDefaultStore().set(engineAtom, engine);

    engine.start();

    return () => {
      engine.destroy();
      getDefaultStore().set(engineAtom, null);
    };
  }, [world]);

  return <Canvas />;
}
