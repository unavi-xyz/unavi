"use client";

import { Client } from "@wired-labs/react-client";
import { ERC721Metadata, getHostFromMetadata } from "contracts";
import Script from "next/script";
import { useState } from "react";
import { useProvider, useSigner } from "wagmi";

import { env } from "@/src/env.mjs";
import { useHotkeys } from "@/src/play/hooks/useHotkeys";

import ClientApp from "./ClientApp";

interface Props {
  id: number;
  metadata: ERC721Metadata;
}

export default function App({ id, metadata }: Props) {
  const [scriptsReady, setScriptsReady] = useState(false);

  const provider = useProvider();
  const { data: signer } = useSigner();

  useHotkeys();

  const host =
    process.env.NODE_ENV === "development"
      ? "localhost:4000"
      : getHostFromMetadata(metadata) ?? env.NEXT_PUBLIC_DEFAULT_HOST;

  return (
    <>
      <Script src="/scripts/draco_decoder.js" onReady={() => setScriptsReady(true)} />

      <div className="fixed h-screen w-screen">
        {scriptsReady && (
          <Client
            spaceId={id}
            metadata={metadata}
            host={host}
            animations="/models"
            defaultAvatar="/models/Robot.vrm"
            ethers={signer ?? provider}
            skybox="/images/Skybox.jpg"
          >
            <ClientApp spaceId={id} metadata={metadata} />
          </Client>
        )}
      </div>
    </>
  );
}
