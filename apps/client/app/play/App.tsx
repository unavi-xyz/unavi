"use client";

import { WorldMetadata } from "@wired-protocol/types";
import dynamic from "next/dynamic";
import Script from "next/script";
import { useEffect, useState } from "react";

import { HOME_SERVER } from "@/src/constants";
import { env } from "@/src/env.mjs";
import { useHotkeys } from "@/src/play/hooks/useHotkeys";
import { useLoadUser } from "@/src/play/hooks/useLoadUser";

import LoadingScreen from "./LoadingScreen";
import { usePlayStore } from "./store";
import { WorldUriId } from "./types";

const Client = dynamic(
  () => import("@unavi/react-client").then((m) => m.Client),
  { ssr: false }
);

const Overlay = dynamic(() => import("./Overlay"), { ssr: false });

interface Props {
  id: WorldUriId;
  metadata: WorldMetadata;
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

  const host =
    process.env.NODE_ENV === "development"
      ? "localhost:4000"
      : metadata.info?.host || env.NEXT_PUBLIC_DEFAULT_HOST;

  return (
    <>
      <Script
        src="/scripts/draco_wasm_wrapper_gltf.js"
        onReady={() => setScriptsReady(true)}
      />

      <LoadingScreen metadata={metadata} />

      <Overlay id={id} metadata={metadata} />

      <div className="fixed h-screen w-screen">
        {scriptsReady && (
          <Client
            uri={uri}
            host={host}
            animations="/models"
            defaultAvatar="/models/Robot.vrm"
            skybox="/images/Skybox.jpg"
            baseHomeServer={HOME_SERVER}
          />
        )}
      </div>
    </>
  );
}
