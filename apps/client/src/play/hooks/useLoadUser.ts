import { useClient } from "@unavi/react-client";
import { useEffect } from "react";

import { usePlayStore } from "@/app/play/store";
import { useAuth } from "@/src/client/AuthProvider";
import { env } from "@/src/env.mjs";

import { LocalStorageKey } from "../constants";

export function useLoadUser() {
  const { user } = useAuth();
  const { send } = useClient();

  // Load nickname from local storage on initial load
  useEffect(() => {
    const localName = localStorage.getItem(LocalStorageKey.Name);
    usePlayStore.setState({ nickname: localName });

    // Send to host
    send({ id: "xyz.unavi.world.user.name", data: localName });
  }, [send]);

  // Publish handle on change
  useEffect(() => {
    const handle = user?.username
      ? `${user.username}@${new URL(env.NEXT_PUBLIC_DEPLOYED_URL).origin}`
      : null;

    // Send to host
    send({ id: "xyz.unavi.world.user.handle", data: handle });
  }, [user, send]);
}
