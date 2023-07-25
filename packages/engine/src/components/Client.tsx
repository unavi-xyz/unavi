import { WorldMetadata } from "@wired-protocol/types";
import { Engine } from "lattice-engine/core";
import { useEffect } from "react";

import { useClientStore } from "../client/clientStore";
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

export function Client({ skybox, defaultAvatar, uri }: Props) {
  const world = useWorld();
  const sendWebSockets = useClientStore((state) => state.sendWebSockets);
  const avatar = useClientStore((state) => state.avatar);
  const handle = useClientStore((state) => state.handle);
  const name = useClientStore((state) => state.name);

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

  useEffect(() => {
    sendWebSockets({
      data: avatar,
      id: "com.wired-protocol.world.user.avatar",
    });
  }, [sendWebSockets, avatar]);

  useEffect(() => {
    sendWebSockets({
      data: handle,
      id: "com.wired-protocol.world.user.handle",
    });
  }, [sendWebSockets, handle]);

  useEffect(() => {
    sendWebSockets({
      data: name,
      id: "com.wired-protocol.world.user.name",
    });
  }, [sendWebSockets, name]);

  return <Canvas />;
}