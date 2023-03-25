import { useEngine, useWebSocket } from "@wired-labs/react-client";
import { useEffect } from "react";

import { usePlayStore } from "../../../app/play/[id]/store";
import { useSession } from "../../client/auth/useSession";
import { LocalStorageKey } from "../constants";

export function useLoadUser() {
  const { data: session } = useSession();
  const engine = useEngine();
  const { send } = useWebSocket();

  // Load nickname from local storage on initial load
  useEffect(() => {
    const localName = localStorage.getItem(LocalStorageKey.Name);
    usePlayStore.setState({ nickname: localName });

    // Publish to host
    send({ subject: "set_name", data: localName });
  }, [send]);

  // Load avatar from local storage on initial load
  useEffect(() => {
    if (!engine) return;

    const localAvatar = localStorage.getItem(LocalStorageKey.Avatar);
    usePlayStore.setState({ avatar: localAvatar });

    // Update engine
    engine.render.send({ subject: "set_user_avatar", data: localAvatar });

    // Publish to host
    send({ subject: "set_avatar", data: localAvatar });
  }, [engine, send]);

  // Publish address on change
  useEffect(() => {
    const address = session?.address ?? null;
    send({ subject: "set_address", data: address });
  }, [session, send]);
}
