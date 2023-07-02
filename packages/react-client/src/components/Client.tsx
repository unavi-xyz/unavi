import { WorldMetadata } from "@wired-protocol/types";
import { useEffect } from "react";

import { ClientSchedules } from "../constants";
import { useEngine } from "../hooks/useEngine";
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

export function Client({ skybox, uri }: Props) {
  const world = useWorld();
  const engine = useEngine(world);

  useEffect(() => {
    import("../config").then(({ config }) => (config.skyboxUri = skybox ?? ""));
  }, [skybox]);

  useEffect(() => {
    useClientStore.setState({ worldUri: uri });
    if (engine) engine.queueSchedule(ClientSchedules.JoinWorld);
  }, [engine, uri]);

  return <Canvas />;
}
