import { useEffect } from "react";

import { useLens } from "../../client/lens/hooks/useLens";
import { useAppStore } from "../store";

export function useSetIdentity() {
  const { handle } = useLens();

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
