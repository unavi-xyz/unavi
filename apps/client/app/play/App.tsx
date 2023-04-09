"use client";

import { Client } from "@wired-labs/react-client";
import Script from "next/script";
import { useState } from "react";
import { useProvider, useSigner } from "wagmi";

import { useHotkeys } from "@/src/play/hooks/useHotkeys";
import { SpaceMetadata } from "@/src/server/helpers/readSpaceMetadata";

import ClientApp from "./ClientApp";
import { SpaceUriId } from "./types";

interface Props {
  id: SpaceUriId;
  metadata: SpaceMetadata;
}

export default function App({ id, metadata }: Props) {
  const [scriptsReady, setScriptsReady] = useState(false);

  const provider = useProvider();
  const { data: signer } = useSigner();

  useHotkeys();

  return (
    <>
      <Script src="/scripts/draco_decoder.js" onReady={() => setScriptsReady(true)} />

      <div className="fixed h-screen w-screen">
        {scriptsReady && (
          <Client
            uri={metadata.uri}
            host={metadata.host}
            animations="/models"
            defaultAvatar="/models/Robot.vrm"
            ethers={signer ?? provider}
            skybox="/images/Skybox.jpg"
          >
            <ClientApp id={id} metadata={metadata} />
          </Client>
        )}
      </div>
    </>
  );
}
