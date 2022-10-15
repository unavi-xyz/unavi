import { useEntity } from "../../hooks/useEntity";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import { useEditorStore } from "../../store";
import EntityComponents from "./EntityComponents";
import TransformComponent from "./TransformComponent";

export default function InspectMenu() {
  const selectedId = useEditorStore((state) => state.selectedId);
  const name$ = useEntity(selectedId, (entity) => entity.name$);
  const name = useSubscribeValue(name$);

  if (!selectedId) return null;

  return (
    <div className="space-y-4 p-4" key={selectedId}>
      <div className="flex justify-center text-2xl font-bold">
        {name || selectedId.slice(0, 8)}
      </div>
      <div className="space-y-8">
        <TransformComponent entityId={selectedId} />
        <EntityComponents entityId={selectedId} />
      </div>
    </div>
  );
}
