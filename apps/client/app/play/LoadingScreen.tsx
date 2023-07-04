import { useClientStore, useLoadingStore } from "@unavi/react-client";
import { useEffect, useState } from "react";

const LOADING_DELAY = 1000;
const FADE_DURATION = 1000;

export default function LoadingScreen() {
  const loading = useLoadingStore((state) => state.loading);
  const loaded = useLoadingStore((state) => state.loaded);
  const total = useLoadingStore((state) => state.total);
  const message = useLoadingStore((state) => state.message);
  const playerId = useClientStore((state) => state.playerId);

  const [doneLoading, setDoneLoading] = useState(false);
  const [hideScreen, setHideScreen] = useState(false);

  // If nothing is loading after a delay, we're done loading
  useEffect(() => {
    // Wait for us to connect to the server
    if (playerId === null) return;

    const timeout = setTimeout(() => {
      if (loading === 0) {
        setDoneLoading(true);
      }
    }, LOADING_DELAY);

    return () => clearTimeout(timeout);
  }, [playerId, loading, setDoneLoading]);

  // Fade out the loading screen
  useEffect(() => {
    if (!doneLoading) return;

    const timeout = setTimeout(() => {
      setHideScreen(true);
    }, FADE_DURATION);

    return () => clearTimeout(timeout);
  }, [doneLoading, setHideScreen]);

  if (hideScreen) return null;

  const defaultMessage = playerId === null ? "Connecting..." : "Loading...";

  return (
    <div
      className={`fixed z-50 flex h-screen w-screen flex-col items-center justify-center bg-neutral-800 text-white transition duration-1000 ${
        doneLoading ? "pointer-events-none opacity-0" : ""
      }`}
    >
      <div>{message || defaultMessage}</div>
      {total === 0 ? null : (
        <div>
          {loaded} / {total}
        </div>
      )}
    </div>
  );
}
