import { useClient } from "@unavi/react-client";
import { useEffect } from "react";

import { usePlayStore } from "@/app/play/store";

import { useSession } from "../../client/auth/useSession";
import { LocalStorageKey } from "../constants";

export function useLoadUser() {
  const { data: session } = useSession();
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
    const address = session?.address ?? null;
    send({ type: "set_address", data: address });
  }, [session, send]);
}
