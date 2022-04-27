import { Card } from "../../components/base";
import { useLocalScene } from "../../helpers/indexeddb/localScenes/useLocalScene";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const localScene = useLocalScene(id);

  return <Card image={localScene?.image} text={localScene?.name} />;
}
