import { useContext, useEffect } from "react";

import { LensContext } from "@wired-labs/lens";

import { useAppStore } from "../store";

export function useSetIdentity() {
  const { handle } = useContext(LensContext);

  useEffect(() => {
    if (handle) {
      useAppStore.setState({
        identity: {
          isGuest: false,
          handle,
        },
      });
    } else {
      useAppStore.setState({
        identity: {
          isGuest: true,
        },
      });
    }
  }, [handle]);
}
