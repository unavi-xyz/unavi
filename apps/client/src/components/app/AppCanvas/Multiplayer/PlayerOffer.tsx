import { useContext, useEffect, useRef } from "react";

import { Transform } from "./helpers/types";
import { appManager } from "../../helpers/store";
import { SocketContext } from "../../SocketProvider";

import OtherPlayer from "./OtherPlayer";

interface Props {
  id: string;
}

export default function PlayerOffer({ id }: Props) {
  const transformRef = useRef<Transform>();

  const { socket } = useContext(SocketContext);

  useEffect(() => {
    if (!socket) return;
    const connection = new RTCPeerConnection();

    const messageChannel = connection.createDataChannel("message");
    messageChannel.onmessage = (e) => {
      appManager.addMessage(JSON.parse(e.data));
    };
    appManager.addMessageChannel(messageChannel);

    const transformChannel = connection.createDataChannel("transform");
    transformChannel.onmessage = (e) => {
      transformRef.current = JSON.parse(e.data);
    };
    appManager.addTransformChannel(transformChannel);

    connection.onicecandidate = (e) => {
      socket.emit("iceCandidate", id, e.candidate);
    };

    socket.on("iceCandidate", (player, iceCandidate) => {
      if (player !== id) return;
      connection.addIceCandidate(iceCandidate);
    });

    connection.createOffer().then((offer) => {
      connection.setLocalDescription(offer);
      socket.emit("offer", id, offer);
    });

    socket.on("answer", (player, answer) => {
      if (player !== id) return;
      if (!connection.remoteDescription) {
        connection.setRemoteDescription(answer);
        return;
      }

      connection.addIceCandidate(answer);
    });
  }, [id, socket]);

  return <OtherPlayer id={id} transformRef={transformRef} />;
}
