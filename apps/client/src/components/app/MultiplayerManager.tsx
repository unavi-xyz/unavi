import { useWebrtcConnections } from "../../helpers/app/hooks/useWebrtcConnections";
import { usePublishLocation } from "../../helpers/app/hooks/usePublishLocation";

import PlayerAnswer from "./PlayerAnswer";
import PlayerOffer from "./PlayerOffer";

/*
WARNING
the following code is a dumpster fire
it needs a major refactor
i wrote it a while ago and have been too scared to touch it since
it is filled with race conditions and silenced errors that i didnt understand
sorry in advance

i have since gone to church and found god
i will not write code like this again
inshallah 🙏
*/

interface Props {
  spaceId: string;
}

export default function MultiplayerManager({ spaceId }: Props) {
  const { offers, answers } = useWebrtcConnections(spaceId);

  //publish our location
  usePublishLocation();

  return (
    <group>
      {Array.from(offers).map((id) => {
        return <PlayerOffer key={id} id={id} />;
      })}
      {Object.entries(answers).map(([id, offer]) => {
        return <PlayerAnswer key={id} id={id} offer={offer} />;
      })}
    </group>
  );
}
