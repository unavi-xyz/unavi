import { useContext, useEffect, useState } from "react";

import { useExchangeIce } from "../../helpers/app/hooks/useExchangeIce";
import { channelNames, PlayerChannels } from "../../helpers/app/types";
import { SpaceContext } from "./SpaceProvider";

import OtherPlayer from "./OtherPlayer";

interface Props {
  id: string;
}

export default function PlayerOffer({ id }: Props) {
  const [connection, setConnection] = useState<RTCPeerConnection>();
  const [channels, setChannels] = useState<PlayerChannels>();
  const [track, setTrack] = useState<MediaStreamTrack>();

  const {
    socket,
    addChannel,
    addConnection,
    removeConnection,
    createAnswer,
    createOffer,
  } = useContext(SpaceContext);

  useEffect(() => {
    setConnection(new RTCPeerConnection());
  }, []);

  useEffect(() => {
    if (!socket || !connection) return;
    if (connection.signalingState === "closed") return;

    //create channels
    const newChannels: PlayerChannels = {
      identity: connection.createDataChannel("identity"),
      location: connection.createDataChannel("location"),
      message: connection.createDataChannel("message"),
    };
    channelNames.forEach((name) => {
      const channel = newChannels[name];
      addChannel(name, channel);
      newChannels[name] = channel;
    });
    setChannels(newChannels);

    function onAnswer(player: string, answer: RTCSessionDescription) {
      if (player !== id || !connection) return;
      connection.setRemoteDescription(answer).catch(() => {});
    }

    function onOffer(player: string, offer: RTCSessionDescription) {
      if (player !== id || !connection) return;
      connection
        .setRemoteDescription(offer)
        .then(() => {
          createAnswer(connection, id);
        })
        .catch(() => {});
    }

    function onTrack(e: RTCTrackEvent) {
      setTrack(e.track);
    }

    function onNegotiationNeeded() {
      if (!connection) return;
      createOffer(connection, id);
    }

    // if (appManager.track) connection.addTrack(appManager.track);

    addConnection(connection);

    socket.on("answer", onAnswer);
    socket.on("offer", onOffer);
    connection.addEventListener("track", onTrack);
    connection.addEventListener("negotiationneeded", onNegotiationNeeded);

    createOffer(connection, id);

    return () => {
      socket.off("answer", onAnswer);
      socket.off("offer", onOffer);
      connection.removeEventListener("track", onTrack);
      connection.removeEventListener("negotiationneeded", onNegotiationNeeded);
      connection.close();

      removeConnection(connection);
    };

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [connection, id, socket]);

  useExchangeIce(connection, id);

  return <OtherPlayer id={id} channels={channels as any} track={track} />;
}
