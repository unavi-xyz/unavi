import { useContext, useEffect } from "react";

import { IdentityResponseSchema, NetworkingContext } from "@wired-xr/engine";

import { useAppStore } from "../store";

export function usePublishIdentity() {
  const { socket } = useContext(NetworkingContext);

  const handle = useAppStore((state) => state.identity.handle);

  useEffect(() => {
    if (!socket) return;

    socket.emit("set_identity", { handle: handle ?? null }, (res) => {
      const { success } = IdentityResponseSchema.parse(res);
      if (!success) {
        console.error("Failed to publish identity");
      }
    });
  }, [socket, handle]);
}
