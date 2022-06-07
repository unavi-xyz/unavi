import { useEffect } from "react";

import { useLensStore } from "../../lens/store";
import { useAppStore } from "../store";

export function useSetIdentity() {
  const handle = useLensStore((state) => state.handle);

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
