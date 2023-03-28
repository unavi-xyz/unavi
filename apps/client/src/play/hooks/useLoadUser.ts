import { useClient } from "@wired-labs/react-client";
import { useEffect } from "react";

import { usePlayStore } from "../../../app/play/[id]/store";
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
    send({ subject: "set_name", data: localName });
  }, [send]);

  // Publish address on change
  useEffect(() => {
    const address = session?.address ?? null;
    send({ subject: "set_address", data: address });
  }, [session, send]);
}
