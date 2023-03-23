import { useEffect } from "react";

import { usePlayStore } from "../../../app/play/[id]/store";
import { useSession } from "../../client/auth/useSession";
import { LocalStorageKey } from "../constants";
import { sendToHost } from "./useHost";
import { usePlayerName } from "./usePlayerName";

export function useLoadUser() {
  const engine = usePlayStore((state) => state.engine);
  const ws = usePlayStore((state) => state.ws);
  const playerId = usePlayStore((state) => state.playerId);
  const playerName = usePlayerName(playerId);
  const { data: session } = useSession();

  // Load nickname from local storage on initial load
  useEffect(() => {
    if (!playerName) return;
    const localName = localStorage.getItem(LocalStorageKey.Name);
    playerName.nickname = localName;
    usePlayStore.setState({ nickname: localName });
  }, [playerName]);

  // Load avatar from local storage on initial load
  useEffect(() => {
    if (!engine || !ws || ws.readyState !== ws.OPEN) return;
    const localAvatar = localStorage.getItem(LocalStorageKey.Avatar);
    usePlayStore.setState({ avatar: localAvatar });
    // Update engine
    engine.render.send({ subject: "set_user_avatar", data: localAvatar });
    // Publish to host
    sendToHost({ subject: "set_avatar", data: localAvatar });
  }, [engine, ws, ws?.readyState]);

  // Publish name on change
  useEffect(() => {
    if (!playerName || !engine || !ws || ws.readyState !== ws.OPEN) return;
    sendToHost({ subject: "set_name", data: playerName.displayName });
  }, [engine, ws, ws?.readyState, playerName, playerName?.displayName]);

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
