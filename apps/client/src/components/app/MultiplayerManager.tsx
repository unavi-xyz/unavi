import { usePublishLocation } from "../../helpers/app/hooks/usePublishLocation";
import { useWebrtcConnections } from "../../helpers/app/hooks/useWebrtcConnections";
import PlayerAnswer from "./PlayerAnswer";
import PlayerOffer from "./PlayerOffer";

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
