import { useEffect } from "react";

import { useAppStore } from "../../app/store";
import { useSession } from "../../client/auth/useSession";
import { LocalStorageKey } from "../constants";
import { sendToHost } from "./useHost";
import { usePlayer } from "./usePlayer";

export function useLoadUser() {
  const engine = useAppStore((state) => state.engine);
  const ws = useAppStore((state) => state.ws);
  const playerId = useAppStore((state) => state.playerId);
  const player = usePlayer(playerId);
  const { data: session } = useSession();

  // Set data on initial load
  useEffect(() => {
    if (!player || !engine || !ws || ws.readyState !== ws.OPEN) return;

    // Set nickname
    const localName = localStorage.getItem(LocalStorageKey.Name);
    if (localName !== player.nickname) {
      sendToHost({ subject: "set_name", data: localName });
    }

    // Set avatar
    const localAvatar = localStorage.getItem(LocalStorageKey.Avatar);

    if (localAvatar) {
      if (localAvatar !== player.avatar) {
        sendToHost({ subject: "set_avatar", data: localAvatar });
      }
    } else {
      // If no avatar set, use default avatar
      sendToHost({ subject: "set_avatar", data: null });
    }
  }, [engine, ws, ws?.readyState, player]);

  // Publish address on change
  useEffect(() => {
    if (!ws || ws.readyState !== ws.OPEN) return;
    const address = session?.address ?? null;
    sendToHost({ subject: "set_address", data: address });
  }, [session, ws, ws?.readyState]);
}
