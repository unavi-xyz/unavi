import { LIGHTS_PRESETS } from "../../../helpers/studio/presets";
import { useStudioStore } from "../../../helpers/studio/store";

export default function LightsMenu() {
  const selectedId = useStudioStore((state) => state.selectedId);
  const addPreset = useStudioStore((state) => state.addPreset);

  return (
    <div className="py-2 space-y-2">
      <div className="px-2 space-y-2">
        {Object.entries(LIGHTS_PRESETS).map(([name, preset]) => (
          <button
            key={name}
            onClick={() => {
              const entity = addPreset(preset, selectedId);
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
