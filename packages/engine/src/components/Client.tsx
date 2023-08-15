import { SetPlayerData } from "@wired-protocol/types";
import { Engine } from "lattice-engine/core";
import { useEffect } from "react";

import { useClientStore } from "../client/clientStore";
import { useWorld } from "../hooks/useWorld";
import Canvas from "./Canvas";

interface Props {
  defaultAvatar?: string;
  skybox?: string;
  uri?: string;
}

export function Client({ skybox, defaultAvatar, uri }: Props) {
  const world = useWorld();
  const sendWebSockets = useClientStore((state) => state.sendWebSockets);
  const avatar = useClientStore((state) => state.avatar);
  const did = useClientStore((state) => state.did);
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

    sendWebSockets({ oneofKind: "setPlayerData", setPlayerData });
  }, [sendWebSockets, avatar]);

  useEffect(() => {
    const setPlayerData = SetPlayerData.create({
      data: {
        did,
      },
    });

    sendWebSockets({ oneofKind: "setPlayerData", setPlayerData });
  }, [sendWebSockets, did]);

  useEffect(() => {
    const setPlayerData = SetPlayerData.create({
      data: {
        nickname,
      },
    });

    sendWebSockets({ oneofKind: "setPlayerData", setPlayerData });
  }, [sendWebSockets, nickname]);

  return <Canvas />;
}
