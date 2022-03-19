import { useLocalScene } from "./localScenes/useLocalScene";
import { Card } from "../base";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const localScene = useLocalScene(id);

  return <Card image={localScene?.image} text={localScene?.name} />;
}
