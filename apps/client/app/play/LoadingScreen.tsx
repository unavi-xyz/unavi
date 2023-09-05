import { connectionStore, loadingStore } from "@unavi/engine";
import { World } from "@wired-protocol/types";
import { useAtom } from "jotai";
import Image from "next/image";
import { useEffect, useState } from "react";

import { env } from "@/src/env.mjs";
import { ClientIdentityProfile } from "@/src/server/helpers/fetchProfile";
import { isFromCDN } from "@/src/utils/isFromCDN";

const LOADING_DELAY = 1000;
const FADE_DURATION = 700;

interface Props {
  metadata: World;
  authors: Array<ClientIdentityProfile | string>;
}

export default function LoadingScreen({ metadata, authors }: Props) {
  const [playerId] = useAtom(connectionStore.playerId);
  const [numLoading] = useAtom(loadingStore.numLoading);
  const [numLoaded] = useAtom(loadingStore.numLoaded);

  const numTotal = numLoading + numLoaded;

  const [doneLoading, setDoneLoading] = useState(false);
  const [hideScreen, setHideScreen] = useState(false);

  // Reset loading state when we unmount
  useEffect(() => {
    return () => {
      loadingStore.set(loadingStore.numLoaded, 0);
    };
  }, []);

  // If nothing is loading after a delay, we're done loading
  useEffect(() => {
    // Wait for us to connect to the server
    if (playerId === null) return;

    const timeout = setTimeout(() => {
      if (numLoading === 0) {
        setDoneLoading(true);
      }
    }, LOADING_DELAY);

    return () => clearTimeout(timeout);
  }, [playerId, numLoading, setDoneLoading]);

  // Fade out the loading screen
  useEffect(() => {
    if (!doneLoading) return;

    const timeout = setTimeout(() => {
      setHideScreen(true);
    }, FADE_DURATION);

    return () => clearTimeout(timeout);
  }, [doneLoading, setHideScreen]);

  if (hideScreen) return null;

  const isConnecting = playerId === null;
  const loadingMessage =
    isConnecting && numTotal === 0 ? "Connecting" : "Loading world";
  const progress = isConnecting ? 0.1 : numLoaded / numTotal;

  const image = metadata?.image;
  const host = metadata?.host ?? env.NEXT_PUBLIC_DEFAULT_HOST;

  return (
    <div
      className={`fixed z-50 flex h-screen w-screen flex-col items-center justify-center bg-neutral-900 text-white transition duration-700 ${
        doneLoading ? "pointer-events-none opacity-0" : ""
      }`}
    >
      {image ? <BackgroundImage src={image} /> : null}

      <div className="absolute bottom-0 left-0 p-4">
        <div className="flex h-full items-stretch space-x-6">
          <div className="flex w-1/3 min-w-fit flex-col justify-between text-white/50">
            {authors.length > 0 ? <div>By</div> : null}
            <div>Host</div>
          </div>

          <div className="flex w-full flex-col justify-between whitespace-nowrap text-white/70">
            <div>
              {authors.map((author, i) => {
                if (typeof author === "string") {
                  return <div key={i}>{author}</div>;
                }

                switch (author.type) {
                  case "db": {
                    return (
                      <a key={i} target="_blank" href={`/@${author.username}`}>
                        @{author.username}
                      </a>
                    );
                  }
                  case "did": {
                    return (
                      <a key={i} target="_blank" href={`/${author.did}`}>
                        {author.did}
                      </a>
                    );
                  }
                }
              })}
            </div>
            <div>{host}</div>
          </div>
        </div>
      </div>

      <div className="z-20 flex flex-col items-center justify-center">
        <h1 className="text-4xl font-bold">{metadata?.title}</h1>
        <div className="space-y-3 pt-6">
          <LoadingBar progress={progress} />

          <div className="space-x-2 text-center text-white/70">
            <span>{loadingMessage}</span>

            {numTotal === 0 ? null : (
              <span>
                ({numLoaded}/{numTotal})
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function LoadingBar({ progress }: { progress: number }) {
  return (
    <div className="relative h-1 w-48 rounded-full outline outline-2 outline-offset-2 outline-white/20">
      <div
        className="absolute left-0 top-0 h-full animate-backgroundScroll rounded-full bg-gradient-to-r from-amber-400 via-lime-500 to-sky-500 transition-all duration-300"
        style={{ width: `${progress * 100}%` }}
      />
    </div>
  );
}

function BackgroundImage({ src }: { src: string }) {
  return (
    <div className="absolute inset-0 select-none opacity-20 blur-2xl">
      {isFromCDN(src) ? (
        <Image
          src={src}
          draggable={false}
          priority
          fill
          sizes="100vw"
          alt=""
          className="object-cover"
        />
      ) : (
        <img
          src={src}
          draggable={false}
          sizes="100vw"
          alt=""
          className="h-full w-full object-cover"
          crossOrigin="anonymous"
        />
      )}
    </div>
  );
}
