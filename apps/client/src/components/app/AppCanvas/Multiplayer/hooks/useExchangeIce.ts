import { useEffect } from "react";
import { Socket } from "socket.io-client";

export default function useExchangeIce(
  socket: Socket,
  connection: RTCPeerConnection,
  targetId: string
) {
  useEffect(() => {
    if (!socket || !connection || !targetId) return;

    function onCreateIce(e: RTCPeerConnectionIceEvent) {
      if (!e.candidate) return;
      socket.emit("iceCandidate", targetId, e.candidate);
    }

    function onRecieveIce(player: string, iceCandidate: RTCIceCandidate) {
      if (player !== targetId) return;
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
