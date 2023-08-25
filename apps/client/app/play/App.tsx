"use client";

import { World } from "@wired-protocol/types";
import dynamic from "next/dynamic";
import Script from "next/script";
import { Suspense, useEffect, useState } from "react";

import { useHotkeys } from "@/src/play/hooks/useHotkeys";
import { useLoadUser } from "@/src/play/hooks/useLoadUser";
import { ClientIdentityProfile } from "@/src/server/helpers/fetchProfile";

import LoadingScreen from "./LoadingScreen";
import { usePlayStore } from "./playStore";
import { WorldUriId } from "./types";

const Client = dynamic(() => import("./Client"), {
  ssr: false,
});

const Overlay = dynamic(() => import("./Overlay"), { ssr: false });

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

      <Suspense>
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
      </Suspense>
    </>
  );
}
