import { OBJECT_PRESETS } from "../../../helpers/studio/presets";
import { useStudioStore } from "../../../helpers/studio/store";

export default function ObjectsMenu() {
  const selectedId = useStudioStore((state) => state.selectedId);
  const addPreset = useStudioStore((state) => state.addPreset);

  return (
    <div className="p-2 space-y-1">
      {Object.entries(OBJECT_PRESETS).map(([name, preset]) => (
        <button
          key={name}
          onClick={() => {
            const entity = addPreset(preset, selectedId);
            if (entity) useStudioStore.setState({ selectedId: entity.id });
          }}
          className="w-full flex hover:bg-primaryContainer hover:text-onPrimaryContainer
                     rounded-md px-4 py-1 transition items-center space-x-2"
        >
          <div>{name}</div>
        </button>
      ))}
    </div>
  );
}
