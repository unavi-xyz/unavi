import { WorldMetadata } from "@wired-protocol/types";
import { useEffect } from "react";

import { useEngine } from "../hooks/useEngine";
import { useWorld } from "../hooks/useWorld";
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

export function Client({ skybox, uri }: Props) {
  const world = useWorld();
  const engine = useEngine(world);

  useEffect(() => {
    import("../config").then(({ config }) => (config.skyboxUri = skybox ?? ""));
  }, [skybox]);

  useEffect(() => {
    import("../config").then(({ config }) => (config.worldUri = uri ?? ""));
  }, [uri]);

  return <Canvas />;
}
