import { useContext, useEffect, useState } from "react";

import { channelNames, PlayerChannels } from "../../helpers/types";
import { appManager } from "../../helpers/store";
import { SocketContext } from "../../SocketProvider";

import useExchangeIce from "./hooks/useExchangeIce";
import OtherPlayer from "./OtherPlayer";

interface Props {
  id: string;
}

export default function PlayerOffer({ id }: Props) {
  const [connection] = useState(new RTCPeerConnection());
  const [channels, setChannels] = useState<PlayerChannels>({});

  const { socket } = useContext(SocketContext);

  useExchangeIce(socket, connection, id);

  useEffect(() => {
    if (!socket) return;

    //create channels
    const newChannels: PlayerChannels = {};
    channelNames.map((name) => {
      const channel = connection.createDataChannel(name);
      appManager.addChannel(name, channel);
      newChannels[name] = channel;
    });
    setChannels(newChannels);

    //create offer
    connection.createOffer().then((offer) => {
      connection.setLocalDescription(offer);
      socket.emit("offer", id, offer);
    });

    //listen for answer
    function onAnswer(player: string, answer: RTCSessionDescription) {
      if (player !== id) return;
      connection.setRemoteDescription(answer);
    }

    socket.on("answer", onAnswer);
    return () => {
      socket.off("answer", onAnswer);
      connection.close();
    };
  }, [connection, id, socket]);

  return <OtherPlayer id={id} channels={channels} />;
}
