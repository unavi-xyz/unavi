import { useContext, useEffect, useState } from "react";

import { useExchangeIce } from "../../helpers/app/hooks/useExchangeIce";
import { PlayerChannels, channelNames } from "../../helpers/app/types";
import { ConnectionContext } from "./ConnectionProvider";
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
  } = useContext(ConnectionContext);

  useEffect(() => {
    setConnection(new RTCPeerConnection());
  }, []);

  useEffect(() => {
    if (!socket || !connection) return;
    if (connection.signalingState === "closed") return;

    async function onAnswer(player: string, answer: RTCSessionDescription) {
      try {
        if (!connection) throw new Error("Connection is undefined");
        if (player !== id) throw new Error("Wrong player");

        await connection.setRemoteDescription(answer);
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

    //handle events
    socket.on("answer", onAnswer);
    socket.on("offer", onOffer);
    connection.addEventListener("track", onTrack);
    connection.addEventListener("negotiationneeded", onNegotiationNeeded);

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

    //initiate connection
    addConnection(connection);
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

  return <OtherPlayer id={id} channels={channels} track={track} />;
}
