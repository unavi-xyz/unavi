import { useTreeItem } from "./hooks/useTreeItem";
import TreeItem from "./TreeItem";

interface Props {
  rootId: bigint;
}

export default function SceneTree({ rootId }: Props) {
  const rootItem = useTreeItem(rootId);

  if (!rootItem) {
    return null;
  }

  return (
    <div className="space-y-1">
      {rootItem.childrenIds.map((id) => (
        <TreeItem key={id.toString()} id={id} />
      ))}
    </div>
  );
}
