import { useSceneStore } from "@unavi/engine";

import { useTreeValue } from "../../hooks/useTreeValue";
import TreeItem from "./TreeItem";

interface Props {
  rootId: bigint;
}

export default function SceneTree({ rootId }: Props) {
  const childrenIds = useTreeValue(rootId, "childrenIds");

  function clearSelected() {
    useSceneStore.setState({ selectedId: undefined });
  }

  return (
    <div onClick={clearSelected} className="h-full space-y-1">
      {childrenIds?.map((id) => (
        <TreeItem key={id.toString()} id={id} />
      ))}
    </div>
  );
}
