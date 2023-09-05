"use client";

import { World } from "@wired-protocol/types";
import Script from "next/script";
import { useEffect, useState } from "react";

import { useHotkeys } from "@/src/play/hooks/useHotkeys";
import { useLoadUser } from "@/src/play/hooks/useLoadUser";
import { ClientIdentityProfile } from "@/src/server/helpers/fetchProfile";

import Client from "./Client";
import LoadingScreen from "./LoadingScreen";
import Overlay from "./Overlay";
import { usePlayStore } from "./playStore";
import { WorldUriId } from "./types";

interface Props {
  id: WorldUriId;
  metadata: World;
  authors: Array<ClientIdentityProfile | string>;
  uri: string;
}

export default function App({ id, metadata, authors, uri }: Props) {
  const [scriptsReady, setScriptsReady] = useState(false);

  useHotkeys();
  useLoadUser();

  useEffect(() => {
    usePlayStore.setState({ metadata });
  }, [metadata]);

  useEffect(() => {
    usePlayStore.setState({ worldId: id });
  }, [id]);

  return (
    <>
      <Script
        src="/scripts/draco_wasm_wrapper_gltf.js"
        onReady={() => setScriptsReady(true)}
      />

      <LoadingScreen metadata={metadata} authors={authors} />

      <Overlay id={id} metadata={metadata} />

      <div className="fixed h-screen w-screen">
        {scriptsReady ? (
          <Client
            uri={uri}
            defaultAvatar="/models/Robot.vrm"
            skybox="/images/Skybox.jpg"
          />
        ) : null}
      </div>
    </>
  );
}
