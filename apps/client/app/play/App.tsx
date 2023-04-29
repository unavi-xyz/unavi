"use client";

import { Client } from "@unavi/react-client";
import { WorldMetadata } from "@wired-protocol/types";
import Script from "next/script";
import { useState } from "react";

import { env } from "@/src/env.mjs";
import { useHotkeys } from "@/src/play/hooks/useHotkeys";

import ClientApp from "./ClientApp";
import { SpaceUriId } from "./types";

interface Props {
  id: SpaceUriId;
  metadata: WorldMetadata;
  uri: string;
}

export default function App({ id, metadata, uri }: Props) {
  const [scriptsReady, setScriptsReady] = useState(false);

  useHotkeys();

  const host =
    process.env.NODE_ENV === "development"
      ? "localhost:4000"
      : metadata.info?.host || env.NEXT_PUBLIC_DEFAULT_HOST;

  const useOffscreenCanvas =
    typeof OffscreenCanvas !== "undefined" && process.env.NODE_ENV !== "development";

  return (
    <>
      <Script src="/scripts/draco_decoder.js" onReady={() => setScriptsReady(true)} />

      <div className="fixed h-screen w-screen">
        {scriptsReady && (
          <Client
            uri={uri}
            host={host}
            animations="/models"
            defaultAvatar="/models/Robot.vrm"
            skybox="/images/Skybox.jpg"
            engineOptions={{ useOffscreenCanvas }}
          >
            <ClientApp id={id} metadata={metadata} />
          </Client>
        )}
      </div>
    </>
  );
}
