import { useContext, useEffect } from "react";
import { SpaceContext } from "../../../components/app/SpaceProvider";

export function useExchangeIce(
  connection: RTCPeerConnection | undefined,
  targetId: string
) {
  const { socket } = useContext(SpaceContext);

  useEffect(() => {
    if (!socket || !connection || !targetId) return;

    function onCreateIce(e: RTCPeerConnectionIceEvent) {
      if (!e.candidate || !socket) return;
      socket.emit("iceCandidate", targetId, e.candidate);
    }

    function onRecieveIce(player: string, iceCandidate: RTCIceCandidate) {
      if (player !== targetId || !connection) return;
      connection.addIceCandidate(iceCandidate);
    }

    connection.addEventListener("icecandidate", onCreateIce);
    socket.on("iceCandidate", onRecieveIce);
    return () => {
      connection.removeEventListener("icecandidate", onCreateIce);
      socket.off("iceCandidate", onRecieveIce);
    };
  }, [connection, targetId, socket]);
}
