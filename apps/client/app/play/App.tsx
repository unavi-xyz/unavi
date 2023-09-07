"use client";

import { World } from "@wired-protocol/types";
import dynamic from "next/dynamic";
import Script from "next/script";
import { useEffect, useState } from "react";

import { useHotkeys } from "@/src/play/hooks/useHotkeys";
import { useLoadUser } from "@/src/play/hooks/useLoadUser";

import { usePlayStore } from "./playStore";
import { WorldUriId } from "./types";

const Client = dynamic(() => import("@unavi/engine").then((m) => m.Client), {
  ssr: false,
});

const Overlay = dynamic(() => import("./Overlay"), { ssr: false });

interface Props {
  id: WorldUriId;
  metadata: World;
  uri: string;
}

export default function App({ id, metadata, uri }: Props) {
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

      <Overlay id={id} metadata={metadata} />

      <div className="fixed h-screen w-screen">
        {scriptsReady && (
          <Client
            uri={uri}
            defaultAvatar="/models/Robot.vrm"
            skybox="/images/Skybox.jpg"
          />
        )}
      </div>
    </>
  );
}
