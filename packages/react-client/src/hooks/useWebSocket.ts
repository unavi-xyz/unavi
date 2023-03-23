import { ToHostMessage } from "@wired-labs/protocol";
import { useCallback, useContext } from "react";

import { ClientContext } from "../Client";

export function useWebSocket() {
  const { ws } = useContext(ClientContext);

  const send = useCallback(
    (message: ToHostMessage) => {
      if (!ws || ws.readyState !== ws.OPEN) return;
      ws.send(JSON.stringify(message));
    },
    [ws]
  );

  return { ws, send };
}
