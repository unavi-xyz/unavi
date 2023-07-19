import { useSceneStore } from "@unavi/react-client";

import { useTreeItem } from "../../hooks/useTreeItem";
import TreeItem from "./TreeItem";

interface Props {
  rootId: bigint;
}

export default function SceneTree({ rootId }: Props) {
  const rootItem = useTreeItem(rootId);

  if (!rootItem) {
    return null;
  }

  function clearSelected() {
    useSceneStore.setState({ selectedId: undefined });
  }

  return (
    <div onClick={clearSelected} className="h-full space-y-1">
      {rootItem.childrenIds.map((id) => (
        <TreeItem key={id.toString()} id={id} />
      ))}
    </div>
  );
}
