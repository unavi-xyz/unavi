import { useEffect } from "react";
import { useAuth } from "ceramic";

import { appManager } from "../../../helpers/store";
import { Identity } from "../../../helpers/types";

export default function usePublishIdentity() {
  const { authenticated, viewerId } = useAuth();

  useEffect(() => {
    const identity: Identity = { isGuest: !authenticated, did: viewerId };
    appManager.setIdentity(identity);
  }, [authenticated, viewerId]);
}
