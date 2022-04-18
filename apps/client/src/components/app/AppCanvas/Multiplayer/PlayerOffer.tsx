import { useContext, useEffect, useState } from "react";

import { channelNames, PlayerChannels } from "../../helpers/types";
import { appManager, useStore } from "../../helpers/store";
import { SocketContext } from "../../SocketProvider";

import useExchangeIce from "./hooks/useExchangeIce";
import OtherPlayer from "./OtherPlayer";
import { createAnswer, createOffer } from "./helpers/connection";

interface Props {
  id: string;
}

export default function PlayerOffer({ id }: Props) {
  const [connection, setConnection] = useState<RTCPeerConnection>(undefined);
  const [channels, setChannels] = useState<PlayerChannels>({});
  const [track, setTrack] = useState<MediaStreamTrack>(undefined);

  const { socket } = useContext(SocketContext);

  useEffect(() => {
    setConnection(new RTCPeerConnection());
  }, []);

  useEffect(() => {
    if (!socket || !connection) return;
    if (connection.signalingState === "closed") return;

    //create channels
    const newChannels: PlayerChannels = {};
    channelNames.map((name) => {
      const channel = connection.createDataChannel(name);
      appManager.addChannel(name, channel);
      newChannels[name] = channel;
    });
    setChannels(newChannels);

    function onAnswer(player: string, answer: RTCSessionDescription) {
      if (player !== id) return;
      connection.setRemoteDescription(answer);
    }

    function onOffer(player: string, offer: RTCSessionDescription) {
      if (player !== id) return;
      connection.setRemoteDescription(offer).then(() => {
        createAnswer(connection, socket, id);
      });
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

    socket.on("answer", onAnswer);
    socket.on("offer", onOffer);
    connection.addEventListener("track", onTrack);
    connection.addEventListener("negotiationneeded", onNegotiationNeeded);
    return () => {
      socket.off("answer", onAnswer);
      socket.off("offer", onOffer);
      connection.removeEventListener("track", onTrack);
      connection.removeEventListener("negotiationneeded", onNegotiationNeeded);
      connection.close();

      const connections = useStore.getState().connections;
      const newConnections = connections.filter((item) => item !== connection);
      useStore.setState({ connections: newConnections });
    };
  }, [connection, id, socket]);

  useExchangeIce(socket, connection, id);

  return <OtherPlayer id={id} channels={channels} track={track} />;
}
