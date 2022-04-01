import { useEffect } from "react";
import { Socket } from "socket.io-client";

export default function useExchangeIce(
  socket: Socket,
  connection: RTCPeerConnection,
  targetId: string
) {
  useEffect(() => {
    if (!socket || !connection || !targetId) return;

    function onCreateIceCandidate(e: RTCPeerConnectionIceEvent) {
      socket.emit("iceCandidate", targetId, e.candidate);
    }

    function onRecieveIceCandidate(
      player: string,
      iceCandidate: RTCIceCandidate
    ) {
      if (player !== targetId) return;
      connection.addIceCandidate(iceCandidate);
    }

    connection.addEventListener("icecandidate", onCreateIceCandidate);
    socket.on("iceCandidate", onRecieveIceCandidate);
    return () => {
      connection.removeEventListener("icecandidate", onCreateIceCandidate);
      socket.off("iceCandidate", onRecieveIceCandidate);
    };
  }, [connection, targetId, socket]);
}
