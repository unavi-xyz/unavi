import { useEffect } from "react";

import { useAppStore } from "../../app/store";
import { useSession } from "../../client/auth/useSession";
import { LocalStorageKey } from "../constants";
import { sendToHost } from "./useHost";
import { usePlayerName } from "./usePlayerName";

export function useLoadUser() {
  const engine = useAppStore((state) => state.engine);
  const ws = useAppStore((state) => state.ws);
  const playerId = useAppStore((state) => state.playerId);
  const avatar = useAppStore((state) => state.avatar);
  const playerName = usePlayerName(playerId);
  const { data: session } = useSession();

  // Load nickname from local storage on initial load
  useEffect(() => {
    if (!playerName) return;
    const localName = localStorage.getItem(LocalStorageKey.Name);
    playerName.nickname = localName;
    useAppStore.setState({ nickname: localName });
  }, [playerName]);

  // Publish name on change
  useEffect(() => {
    if (!playerName || !engine || !ws || ws.readyState !== ws.OPEN) return;
    sendToHost({ subject: "set_name", data: playerName.displayName });
  }, [engine, ws, ws?.readyState, playerName, playerName?.displayName]);

  // Publish avatar on change
  useEffect(() => {
    if (!ws || ws.readyState !== ws.OPEN) return;

    if (avatar) {
      sendToHost({ subject: "set_avatar", data: avatar });
      return;
    }

    const localAvatar = localStorage.getItem(LocalStorageKey.Avatar);
    if (localAvatar) {
      sendToHost({ subject: "set_avatar", data: localAvatar });
      return;
    }

    sendToHost({ subject: "set_avatar", data: null });
  }, [avatar, ws, ws?.readyState]);

  // Publish address on change
  useEffect(() => {
    if (!ws || ws.readyState !== ws.OPEN) return;
    const address = session?.address ?? null;
    sendToHost({ subject: "set_address", data: address });
  }, [session, ws, ws?.readyState]);

  // Update address on change
  useEffect(() => {
    if (!playerName || !ws || ws.readyState !== ws.OPEN) return;
    playerName.address = session?.address ?? null;
  }, [playerName, session, ws, ws?.readyState]);
}
