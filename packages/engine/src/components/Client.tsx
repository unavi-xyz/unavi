import { SetPlayerData, World } from "@wired-protocol/types";
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
  metadata?: World;
}

export function Client({ skybox, defaultAvatar, uri }: Props) {
  const world = useWorld();
  const sendWebSockets = useClientStore((state) => state.sendWebSockets);
  const avatar = useClientStore((state) => state.avatar);
  const handle = useClientStore((state) => state.handle);
  const nickname = useClientStore((state) => state.nickname);

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
    const setPlayerData = SetPlayerData.create({
      data: {
        avatar,
      },
    });

    sendWebSockets(SetPlayerData.toBinary(setPlayerData));
  }, [sendWebSockets, avatar]);

  useEffect(() => {
    const setPlayerData = SetPlayerData.create({
      data: {
        handle,
      },
    });

    sendWebSockets(SetPlayerData.toBinary(setPlayerData));
  }, [sendWebSockets, handle]);

  useEffect(() => {
    const setPlayerData = SetPlayerData.create({
      data: {
        nickname,
      },
    });

    sendWebSockets(SetPlayerData.toBinary(setPlayerData));
  }, [sendWebSockets, nickname]);

  return <Canvas />;
}
