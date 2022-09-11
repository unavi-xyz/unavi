import { useEditorStore } from "../../store";
import EntityComponents from "./EntityComponents";
import TransformComponent from "./TransformComponent";

export default function InspectMenu() {
  const selectedId = useEditorStore((state) => state.selectedId);
  const name = useEditorStore((state) => {
    if (!selectedId) return null;
    return state.scene.entities[selectedId].name;
  });

  if (!selectedId || !name) return null;

  return (
    <div className="p-4 space-y-4" key={selectedId}>
      <div className="flex justify-center text-2xl font-bold">{name}</div>
      <div className="space-y-8">
        <TransformComponent entityId={selectedId} />
        <EntityComponents entityId={selectedId} />
      </div>
    </div>
  );
}
