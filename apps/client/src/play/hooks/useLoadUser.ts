import { useEffect } from "react";

import { usePlayStore } from "@/app/play/store";
import { useAuth } from "@/src/client/AuthProvider";
import { HOME_SERVER } from "@/src/constants";

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
    const handle = user?.username ? `${user.username}@${HOME_SERVER}` : null;

    // Send to host
    // send({ data: handle, id: "xyz.unavi.world.user.handle" });
  }, [user]);
}
