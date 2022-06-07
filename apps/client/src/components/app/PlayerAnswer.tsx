import { useContext, useEffect, useState } from "react";

import { useExchangeIce } from "../../helpers/app/hooks/useExchangeIce";
import {
  Channels,
  PlayerChannels,
  channelNames,
} from "../../helpers/app/types";
import { ConnectionContext } from "./ConnectionProvider";
import OtherPlayer from "./OtherPlayer";

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
  } = useContext(ConnectionContext);

  useExchangeIce(connection, id);

  useEffect(() => {
    setConnection(new RTCPeerConnection());
  }, [offer]);

  useEffect(() => {
    if (!socket || !connection) return;
    if (connection.signalingState === "closed") return;

    async function answer() {
      if (!connection) throw new Error("Connection is undefined");

      await connection.setRemoteDescription(offer);
      await createAnswer(connection, id);
    }

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

    async function onNegotiationNeeded() {
      try {
        if (!connection) throw new Error("Connection is undefined");
        await createOffer(connection, id);
      } catch (e) {
        console.error(e);
      }
    }

    async function onOffer(player: string, offer: RTCSessionDescription) {
      try {
        if (!connection) throw new Error("Connection is undefined");
        if (player !== id) throw new Error("Wrong player");

        await connection.setRemoteDescription(offer);
        await createAnswer(connection, id);
      } catch (e) {
        console.error(e);
      }
    }

    //handle events
    socket.on("offer", onOffer);
    connection.addEventListener("datachannel", onDataChannel);
    connection.addEventListener("track", onTrack);
    connection.addEventListener("negotiationneeded", onNegotiationNeeded);

    //answer the offer
    answer();
    addConnection(connection);

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

  return <OtherPlayer id={id} channels={channels} track={track} />;
}
