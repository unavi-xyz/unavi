import { useEditorStore } from "../../store";
import EntityComponent from "./EntityComponent";
import TransformComponent from "./TransformComponent";

export default function InspectMenu() {
  const selectedId = useEditorStore((state) => state.selectedId);
  const tree = useEditorStore((state) => state.tree);
  const name = selectedId ? tree[selectedId].name : null;

  if (!selectedId || !name) return null;

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-center text-2xl font-bold">{name}</div>
      <div className="space-y-8">
        <TransformComponent entityId={selectedId} />
        <EntityComponent entityId={selectedId} />
      </div>
    </div>
  );
}
