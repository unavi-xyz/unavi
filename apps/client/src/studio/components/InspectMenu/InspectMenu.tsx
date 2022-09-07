import { useStudioStore } from "../../store";
import TransformComponent from "./TransformComponent";

export default function InspectMenu() {
  const selectedId = useStudioStore((state) => state.selectedId);
  const tree = useStudioStore((state) => state.tree);
  const selected = selectedId ? tree[selectedId] : null;

  if (!selectedId || !selected) return null;

  return (
    <div className="pl-3 pr-4 pt-4 space-y-4">
      <div className="flex justify-center text-2xl font-bold">
        {selected.name}
      </div>

      <div>
        <TransformComponent entityId={selectedId} />
      </div>
    </div>
  );
}
