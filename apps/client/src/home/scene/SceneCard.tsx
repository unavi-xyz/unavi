import { Card } from "../../components/base";
import { useLocalSpace } from "../../helpers/indexedDB/localSpaces/hooks/useLocalScene";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const localScene = useLocalSpace(id);

  return <Card image={localScene?.image} text={localScene?.name} />;
}
