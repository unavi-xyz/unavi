import { useContext, useEffect, useState } from "react";

import { channelNames, Channels, PlayerChannels } from "../../helpers/types";
import { appManager } from "../../helpers/store";
import { SocketContext } from "../../SocketProvider";

import useExchangeIce from "./hooks/useExchangeIce";
import OtherPlayer from "./OtherPlayer";

interface Props {
  id: string;
  offer: RTCSessionDescription;
}

export default function PlayerAnswer({ id, offer }: Props) {
  const [connection] = useState(new RTCPeerConnection());
  const [channels, setChannels] = useState<PlayerChannels>({});

  const { socket } = useContext(SocketContext);

  useExchangeIce(socket, connection, id);

  useEffect(() => {
    if (!socket) return;

    //create answer
    connection.setRemoteDescription(offer).then(async () => {
      const answer = await connection.createAnswer();
      connection.setLocalDescription(answer);
      socket.emit("answer", id, answer);
    });

    //listen for channels
    function onDataChannel(e: RTCDataChannelEvent) {
      const label = e.channel.label as keyof Channels;
      if (channelNames.includes(label)) {
        appManager.addChannel(label, e.channel);
        setChannels((prev) => {
          const clone = { ...prev };
          clone[label] = e.channel;
          return clone;
        });
      }
    }

    connection.addEventListener("datachannel", onDataChannel);
    return () => {
      connection.removeEventListener("datachannel", onDataChannel);
    };
  }, [connection, id, offer, socket]);

  return <OtherPlayer id={id} channels={channels} />;
}
