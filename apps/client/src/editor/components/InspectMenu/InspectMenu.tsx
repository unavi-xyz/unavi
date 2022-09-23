import { useEntity } from "../../hooks/useEntity";
import { useEditorStore } from "../../store";
import EntityComponents from "./EntityComponents";
import TransformComponent from "./TransformComponent";

export default function InspectMenu() {
  const selectedId = useEditorStore((state) => state.selectedId);
  const name = useEntity(selectedId, (entity) => entity.name);

  if (!selectedId || !name) return null;

  return (
    <div className="space-y-4 p-4" key={selectedId}>
      <div className="flex justify-center text-2xl font-bold">{name}</div>
      <div className="space-y-8">
        <TransformComponent entityId={selectedId} />
        <EntityComponents entityId={selectedId} />
      </div>
    </div>
  );
}
