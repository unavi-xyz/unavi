import { useSceneStore } from "@unavi/engine";

import { useNodeValue } from "../../hooks/useNodeValue";
import TreeItem from "./TreeItem";

interface Props {
  rootId: bigint;
}

export default function SceneTree({ rootId }: Props) {
  const childrenIds = useNodeValue(rootId, "childrenIds");

  function clearSelected() {
    useSceneStore.setState({ selectedId: undefined });
  }

  return (
    <div onClick={clearSelected} className="h-full space-y-1">
      {childrenIds?.map((entityId) => (
        <TreeItem key={entityId.toString()} entityId={entityId} />
      ))}
    </div>
  );
}
