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

    // Publish to host
    send({ id: "xyz.unavi.world.user.name", data: localName });
  }, [send]);

  // Publish handle on change
  useEffect(() => {
    const handle = user?.username ? `${user.username}@${env.NEXT_PUBLIC_DEPLOYED_URL}` : null;
    send({ id: "xyz.unavi.world.user.handle", data: handle });
  }, [user, send]);
}
