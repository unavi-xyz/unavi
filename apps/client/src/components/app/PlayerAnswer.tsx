import { useContext, useEffect, useState } from "react";
import { useExchangeIce } from "../../helpers/app/hooks/useExchangeIce";
import { useAppStore } from "../../helpers/app/store";
import {
  channelNames,
  Channels,
  PlayerChannels,
} from "../../helpers/app/types";

import OtherPlayer from "./OtherPlayer";
import { SpaceContext } from "./SpaceProvider";

interface Props {
  id: string;
  offer: RTCSessionDescription;
}

export default function PlayerAnswer({ id, offer }: Props) {
  const [connection, setConnection] = useState<RTCPeerConnection>();
  const [channels, setChannels] = useState<Partial<PlayerChannels>>();
  const [track, setTrack] = useState<MediaStreamTrack>();

  const {
    socket,
    addChannel,
    addConnection,
    removeConnection,
    createAnswer,
    createOffer,
  } = useContext(SpaceContext);

  useExchangeIce(connection, id);

  useEffect(() => {
    setConnection(new RTCPeerConnection());
  }, [offer]);

  useEffect(() => {
    if (!socket || !connection) return;
    if (connection.signalingState === "closed") return;

    //create initial answer
    connection
      .setRemoteDescription(offer)
      .then(() => {
        createAnswer(connection, id);
      })
      .catch(() => {});

    function onDataChannel(e: RTCDataChannelEvent) {
      const label = e.channel.label as keyof Channels;
      if (channelNames.includes(label)) {
        addChannel(label, e.channel);
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
      if (!connection) return;
      createOffer(connection, id);
    }

    // if (appManager.track) connection.addTrack(appManager.track);

    addConnection(connection);

    function onOffer(player: string, offer: RTCSessionDescription) {
      if (player !== id || !connection) return;
      connection
        .setRemoteDescription(offer)
        .then(() => {
          createAnswer(connection, id);
        })
        .catch(() => {});
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

      removeConnection(connection);
    };

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [connection, id, offer, socket]);

  return <OtherPlayer id={id} channels={channels as any} track={track} />;
}
