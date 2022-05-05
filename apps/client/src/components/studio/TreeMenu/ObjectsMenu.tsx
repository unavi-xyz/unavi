import { ENTITY_PRESETS } from "../../../helpers/studio/presets";
import { useStudioStore } from "../../../helpers/studio/store";

export default function ObjectsMenu() {
  const selectedId = useStudioStore((state) => state.selectedId);
  const addPreset = useStudioStore((state) => state.addPreset);

  return (
    <div className="py-2 space-y-2">
      <div className="px-2 space-y-2">
        {Object.keys(ENTITY_PRESETS).map((primitive) => (
          <button
            key={primitive}
            onClick={() => {
              const entity = addPreset(primitive, selectedId);
              useStudioStore.setState({ selectedId: entity.id });
            }}
            className="w-full flex hover:bg-neutral-100 rounded-md px-3 py-1 cursor-default"
          >
            {primitive}
          </button>
        ))}
      </div>
    </div>
  );
}
