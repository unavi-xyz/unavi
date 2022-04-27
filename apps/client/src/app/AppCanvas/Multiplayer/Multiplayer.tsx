import usePublishPosition from "./hooks/usePublishPosition";
import usePublishIdentity from "./hooks/usePublishIdentity";
import useConnections from "./hooks/useConnections";

import PlayerAnswer from "./PlayerAnswer";
import PlayerOffer from "./PlayerOffer";

export default function Multiplayer() {
  usePublishPosition();
  usePublishIdentity();

  const { offers, answers } = useConnections();

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
