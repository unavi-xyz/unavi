import { ENTITY_PRESETS } from "../../../helpers/studio/presets";
import { useStudioStore } from "../../../helpers/studio/store";

export default function ObjectsMenu() {
  const selectedId = useStudioStore((state) => state.selectedId);
  const addPreset = useStudioStore((state) => state.addPreset);

  return (
    <div className="py-2 space-y-2">
      <div className="px-2 space-y-2">
        {Object.keys(ENTITY_PRESETS).map((name) => (
          <button
            key={name}
            onClick={() => {
              const entity = addPreset(name, selectedId);
              useStudioStore.setState({ selectedId: entity.id });
            }}
            className="w-full flex hover:bg-primaryContainer hover:text-onPrimaryContainer
                       rounded-md px-4 py-1 transition items-center space-x-2"
          >
            <div>{name}</div>
          </button>
        ))}
      </div>
    </div>
  );
}
