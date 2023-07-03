import { WorldMetadata } from "@wired-protocol/types";
import { Engine } from "lattice-engine/core";
import { useEffect } from "react";

import { useWorld } from "../hooks/useWorld";
import { useClientStore } from "../store";
import Canvas from "./Canvas";

interface Props {
  animations?: string;
  children?: React.ReactNode;
  defaultAvatar?: string;
  host?: string;
  skybox?: string;
  uri?: string;
  baseHomeServer?: string;
  metadata?: WorldMetadata;
}

export function Client({ skybox, defaultAvatar, uri }: Props) {
  const world = useWorld();

  useEffect(() => {
    if (!world) return;

    const engine = new Engine(world);
    useClientStore.setState({ engine });

    engine.start();

    return () => {
      engine.destroy();
      useClientStore.setState({ engine: null });
    };
  }, [world]);

  useEffect(() => {
    useClientStore.setState({ skybox });
  }, [skybox]);

  useEffect(() => {
    useClientStore.setState({ worldUri: uri });
  }, [uri]);

  useEffect(() => {
    useClientStore.setState({ defaultAvatar });
  }, [defaultAvatar]);

  useEffect(() => {
    return () => {
      useClientStore.getState().cleanupConnection();
    };
  }, []);

  return <Canvas />;
}
