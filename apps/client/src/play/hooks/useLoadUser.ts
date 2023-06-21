import { useEffect } from "react";

import { usePlayStore } from "@/app/play/store";
import { useAuth } from "@/src/client/AuthProvider";
import { env } from "@/src/env.mjs";

import { LocalStorageKey } from "../constants";

export function useLoadUser() {
  const { user } = useAuth();

  // Load nickname from local storage on initial load
  useEffect(() => {
    const localName = localStorage.getItem(LocalStorageKey.Name);
    usePlayStore.setState({ nickname: localName });

    // Send to host
    // send({ data: localName, id: "xyz.unavi.world.user.name" });
  }, []);

  // Publish handle on change
  useEffect(() => {
    const handle = user?.username
      ? `${user.username}@${new URL(env.NEXT_PUBLIC_DEPLOYED_URL).origin}`
      : null;

    // Send to host
    // send({ data: handle, id: "xyz.unavi.world.user.handle" });
  }, [user]);
}
