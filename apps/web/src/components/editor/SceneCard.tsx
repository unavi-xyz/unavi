import { useLocalWorld } from "../../helpers/localWorlds/useLocalWorld";
import Card from "../base/Card";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const world = useLocalWorld(id);

  return (
    <div className="w-64 h-40 hover:-translate-y-2 transition-all">
      <Card text={world?.name} image={world?.image} />
    </div>
  );
}
