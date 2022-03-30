import { useContext, useEffect, useRef } from "react";

import { Transform } from "./helpers/types";
import { appManager } from "../../helpers/store";
import { SocketContext } from "../../SocketProvider";

import OtherPlayer from "./OtherPlayer";

interface Props {
  id: string;
  offer: RTCSessionDescription;
}

export default function PlayerAnswer({ id, offer }: Props) {
  const transformRef = useRef<Transform>();

  const { socket } = useContext(SocketContext);

  useEffect(() => {
    if (!socket) return;
    const connection = new RTCPeerConnection();

    connection.ondatachannel = (e) => {
      switch (e.channel.label) {
        case "transform":
          e.channel.onmessage = (e) => {
            transformRef.current = JSON.parse(e.data);
          };
          appManager.addTransformChannel(e.channel);
          break;
        case "message":
          e.channel.onmessage = (e) => {
            appManager.addMessage(JSON.parse(e.data));
          };
          appManager.addMessageChannel(e.channel);
          break;
      }
    };

    connection.onicecandidate = (e) => {
      socket.emit("iceCandidate", id, e.candidate);
    };

    socket.on("iceCandidate", (player, iceCandidate) => {
      if (player !== id) return;
      connection.addIceCandidate(iceCandidate);
    });

    connection.setRemoteDescription(offer).then(async () => {
      const answer = await connection.createAnswer();
      connection.setLocalDescription(answer);
      socket.emit("answer", id, answer);
    });
  }, [id, offer, socket]);

  return <OtherPlayer id={id} transformRef={transformRef} />;
}
