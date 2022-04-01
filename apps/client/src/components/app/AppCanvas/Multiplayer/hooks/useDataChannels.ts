import { useEffect, useRef, useState } from "react";
import { useProfile } from "ceramic";

import { createMessage } from "../../../helpers/message";
import { appManager } from "../../../helpers/store";
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
      message.username = profile?.name ?? id;
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

  return { transformRef, identity };
}
