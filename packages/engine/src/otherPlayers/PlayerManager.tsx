import { useContext } from "react";

import { NetworkingContext } from "../networking";
import { OtherPlayer } from "./OtherPlayer";

interface Props {
  animationsUrl: string;
  defaultAvatarUrl: string;
}

export function PlayerManager({ animationsUrl, defaultAvatarUrl }: Props) {
  const { otherPlayers } = useContext(NetworkingContext);

  return (
    <>
      {Object.keys(otherPlayers).map((playerId) => {
        return (
          <OtherPlayer
            key={playerId}
            id={playerId}
            animationsUrl={animationsUrl}
            defaultAvatarUrl={defaultAvatarUrl}
          />
        );
      })}
    </>
  );
}
