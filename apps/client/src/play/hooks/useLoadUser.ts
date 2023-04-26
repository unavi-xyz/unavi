import { useClient } from "@unavi/react-client";
import { useEffect } from "react";

import { usePlayStore } from "@/app/play/store";
import { useAuth } from "@/src/client/AuthProvider";

import { LocalStorageKey } from "../constants";

export function useLoadUser() {
  const { user } = useAuth();
  const { send } = useClient();

  // Load nickname from local storage on initial load
  useEffect(() => {
    const localName = localStorage.getItem(LocalStorageKey.Name);
    usePlayStore.setState({ nickname: localName });

    // Publish to host
    send({ type: "set_name", data: localName });
  }, [send]);

  // Publish address on change
  useEffect(() => {
    const address = user?.address ?? null;
    send({ type: "set_address", data: address });
  }, [user, send]);
}
