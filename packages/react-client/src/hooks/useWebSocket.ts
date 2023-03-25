import { ToHostMessage } from "@wired-labs/protocol";
import { useCallback, useContext } from "react";

import { ClientContext } from "../components/Client";

export function useWebSocket() {
  const { ws } = useContext(ClientContext);

  const send = useCallback(
    (message: ToHostMessage) => {
      if (!ws || ws.readyState !== ws.OPEN) return;
      ws.send(JSON.stringify(message));
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [ws, ws?.readyState]
  );

  return { ws, send };
}
