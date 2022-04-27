import { useContext, useEffect, useState } from "react";

import { channelNames, Channels, PlayerChannels } from "../../helpers/types";
import { appManager, useStore } from "../../helpers/store";
import { SocketContext } from "../../SocketProvider";

import useExchangeIce from "./hooks/useExchangeIce";
import OtherPlayer from "./OtherPlayer";
import { createAnswer, createOffer } from "./helpers/connection";

interface Props {
  id: string;
  offer: RTCSessionDescription;
}

export default function PlayerAnswer({ id, offer }: Props) {
  const [connection, setConnection] = useState<RTCPeerConnection>(undefined);
  const [channels, setChannels] = useState<PlayerChannels>({});
  const [track, setTrack] = useState<MediaStreamTrack>(undefined);

  const { socket } = useContext(SocketContext);

  useExchangeIce(socket, connection, id);

  useEffect(() => {
    setConnection(new RTCPeerConnection());
  }, [offer]);

  useEffect(() => {
    if (!socket || !connection) return;
    if (connection.signalingState === "closed") return;

    //create initial answer
    connection.setRemoteDescription(offer).then(() => {
      createAnswer(connection, socket, id);
    });

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

    function onTrack(e: RTCTrackEvent) {
      setTrack(e.track);
    }

    function onNegotiationNeeded() {
      createOffer(connection, socket, id);
    }

    if (appManager.track) connection.addTrack(appManager.track);

    const connections = useStore.getState().connections;
    const newConnections = [...connections, connection];
    useStore.setState({ connections: newConnections });

    function onOffer(player: string, offer: RTCSessionDescription) {
      if (player !== id) return;
      connection.setRemoteDescription(offer).then(() => {
        createAnswer(connection, socket, id);
      });
    }

    socket.on("offer", onOffer);
    connection.addEventListener("datachannel", onDataChannel);
    connection.addEventListener("track", onTrack);
    connection.addEventListener("negotiationneeded", onNegotiationNeeded);
    return () => {
      socket.off("offer", onOffer);
      connection.removeEventListener("datachannel", onDataChannel);
      connection.removeEventListener("track", onTrack);
      connection.removeEventListener("negotiationneeded", onNegotiationNeeded);
      connection.close();

      const connections = useStore.getState().connections;
      const newConnections = connections.filter((item) => item !== connection);
      useStore.setState({ connections: newConnections });
    };
  }, [connection, id, offer, socket]);

  return <OtherPlayer id={id} channels={channels} track={track} />;
}
