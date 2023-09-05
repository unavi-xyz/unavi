import { connectionStore, defaultAvatarAtom, skyboxAtom } from "@unavi/engine";
import { SetPlayerData } from "@wired-protocol/types";
import { atom, getDefaultStore, useAtom } from "jotai";
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

  const [avatar] = useAtom(connectionStore.avatar);
  const [did] = useAtom(connectionStore.did);
  const [nickname] = useAtom(connectionStore.nickname);

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

  useEffect(() => {
    getDefaultStore().set(skyboxAtom, skybox ?? "");
  }, [skybox]);

  useEffect(() => {
    connectionStore.set(connectionStore.worldUri, uri ?? "");
  }, [uri]);

  useEffect(() => {
    getDefaultStore().set(defaultAvatarAtom, defaultAvatar ?? "");
  }, [defaultAvatar]);

  useEffect(() => {
    return () => {
      connectionStore.closeConnection();
    };
  }, []);

  useEffect(() => {
    const setPlayerData = SetPlayerData.create({
      data: {
        avatar,
      },
    });

    connectionStore.sendWs({ oneofKind: "setPlayerData", setPlayerData });
  }, [avatar]);

  useEffect(() => {
    const setPlayerData = SetPlayerData.create({
      data: {
        did,
      },
    });

    connectionStore.sendWs({ oneofKind: "setPlayerData", setPlayerData });
  }, [did]);

  useEffect(() => {
    const setPlayerData = SetPlayerData.create({
      data: {
        nickname,
      },
    });

    connectionStore.sendWs({ oneofKind: "setPlayerData", setPlayerData });
  }, [nickname]);

  return <Canvas />;
}
