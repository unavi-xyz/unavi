import { useEffect } from "react";
import { useAuth } from "ceramic";

import { Identity } from "../../../helpers/types";
import { appManager, useStore } from "../../../helpers/store";

export default function usePublishIdentity() {
  const { authenticated, viewerId } = useAuth();

  useEffect(() => {
    const identity: Identity = { isGuest: !authenticated, did: viewerId };
    useStore.setState({ identity });
    appManager.publishAll("identity", identity);
  }, [authenticated, viewerId]);
}
