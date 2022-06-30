import { useContext } from "react";

import { NetworkingContext } from "../networking";
import { OtherPlayer } from "./OtherPlayer";

interface Props {
  animationsUrl: string;
  defaultAvatarUrl: string;
}

export function PlayerManager({ animationsUrl, defaultAvatarUrl }: Props) {
  const { dataConsumers } = useContext(NetworkingContext);

  return (
    <>
      {Object.keys(dataConsumers).map((playerId) => {
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
