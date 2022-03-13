import { useLocalWorld } from "../../helpers/localWorlds/useLocalWorld";
import Card from "../base/Card";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const world = useLocalWorld(id);

  return (
    <div className="w-full h-40 hover:-translate-y-1">
      <Card text={world?.name} image={world?.image} />
    </div>
  );
}
