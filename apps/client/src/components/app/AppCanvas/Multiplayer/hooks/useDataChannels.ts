import { useEffect, useRef, useState } from "react";
import { useProfile } from "ceramic";

import { createMessage } from "../../../helpers/message";
import { appManager, useStore } from "../../../helpers/store";
import { Identity, PlayerChannels, Transform } from "../../../helpers/types";

export default function useDataChannels(id: string, channels: PlayerChannels) {
  const transformRef = useRef<Transform>({ position: [0, 0, 0], rotation: 0 });

  const [identity, setIdentity] = useState<Identity>();

  const { profile } = useProfile(identity?.did);

  useEffect(() => {
    if (!channels) return;
    if (!channels.message || !channels.transform || !channels.identity) return;

    function onMessage(e: MessageEvent<string>) {
      const message = createMessage(JSON.parse(e.data));
      message.username = profile?.name ?? `Guest-${id.substring(0, 6)}`;
      appManager.addMessage(message);
    }

    function onTransform(e: MessageEvent<string>) {
      transformRef.current = JSON.parse(e.data);
    }

    function onIdentity(e: MessageEvent<string>) {
      setIdentity(JSON.parse(e.data));
    }

    channels.message.addEventListener("message", onMessage);
    channels.transform.addEventListener("message", onTransform);
    channels.identity.addEventListener("message", onIdentity);
    return () => {
      channels.message.removeEventListener("message", onMessage);
      channels.transform.removeEventListener("message", onTransform);
      channels.identity.removeEventListener("message", onIdentity);
    };
  }, [channels, id, profile]);

  useEffect(() => {
    //add to players list
    const players = useStore.getState().players;
    const newPlayers = { ...players };
    newPlayers[id] = identity;
    useStore.setState({ players: newPlayers });

    return () => {
      //remove from players list
      const players = useStore.getState().players;
      const newPlayers = { ...players };
      delete newPlayers[id];
      useStore.setState({ players: newPlayers });
    };
  }, [id, identity]);

  return { transformRef, identity };
}
