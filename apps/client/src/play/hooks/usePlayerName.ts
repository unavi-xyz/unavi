import { ClientContext } from "@wired-labs/react-client";
import { useContext, useEffect, useState } from "react";

import { getProfileByAddress } from "../../../app/api/profiles/by-address/[address]/helper";
import { usePlayStore } from "../../../app/play/[id]/store";
import { useSession } from "../../client/auth/useSession";
import { toHex } from "../../utils/toHex";

export function usePlayerName(playerId: number | null) {
  const [name, setName] = useState("");
  const { players, playerId: userId } = useContext(ClientContext);

  const userNickname = usePlayStore((state) => state.nickname);
  const { data: session } = useSession();

  useEffect(() => {
    if (playerId === null) {
      setName("");
      return;
    }

    async function getName() {
      if (playerId === null) return;

      let displayName = "";

      // If this is the current user
      if (playerId === userId) {
        const address = session?.address ?? null;

        if (address) {
          const profile = await getProfileByAddress(address);

          if (profile?.handle) displayName = profile.handle.string;
          else if (!userNickname) displayName = address.substring(0, 6);
        }

        if (!displayName && userNickname) displayName = userNickname;
        if (!displayName) displayName = `Guest ${toHex(playerId)}`;

        setName(displayName);
        return;
      }

      // Otherwise, find the player
      const player = players.find((p) => p.id === playerId);

      if (player) {
        if (player.address) {
          const profile = await getProfileByAddress(player.address);

          if (profile?.handle) displayName = profile.handle.string;
          else if (!player.name) displayName = player.address.substring(0, 6);
        }

        if (!displayName && player.name) displayName = player.name;
      }

      if (!displayName) displayName = `Guest ${toHex(playerId)}`;

      setName(displayName);
    }

    getName();
  }, [playerId, players, session, userId, userNickname]);

  return name;
}
