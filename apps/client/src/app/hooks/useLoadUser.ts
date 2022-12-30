import { useEffect } from "react";

import { useSession } from "../../client/auth/useSession";
import { LocalStorageKey } from "../constants";
import { useAppStore } from "../store";
import { sendToHost } from "./useHost";

export function useLoadUser() {
  const engine = useAppStore((state) => state.engine);
  const ws = useAppStore((state) => state.ws);

  const { data: session } = useSession();

  useEffect(() => {
    if (!ws || ws.readyState !== ws.OPEN) return;

    // Set address
    const address = session?.address ?? null;
    sendToHost({ subject: "set_address", data: address });
  }, [session, ws, ws?.readyState]);

  useEffect(() => {
    if (!engine || !ws || ws.readyState !== ws.OPEN) return;

    const { customAvatar, displayName } = useAppStore.getState();

    // Set name
    const localName = localStorage.getItem(LocalStorageKey.Name);

    if (localName !== displayName) {
      useAppStore.setState({ displayName: localName });
      sendToHost({ subject: "set_name", data: localName });
    }

    // Set avatar
    const localAvatar = localStorage.getItem(LocalStorageKey.Avatar);

    if (localAvatar) {
      if (localAvatar !== customAvatar) {
        useAppStore.setState({ customAvatar: localAvatar });
        engine.renderThread.postMessage({ subject: "set_avatar", data: localAvatar });
        sendToHost({ subject: "set_avatar", data: localAvatar });
      }
    } else {
      // If no avatar set, use default avatar
      useAppStore.setState({ customAvatar: null });
      engine.renderThread.postMessage({ subject: "set_avatar", data: null });
      sendToHost({ subject: "set_avatar", data: null });
    }
  }, [engine, ws, ws?.readyState]);
}
