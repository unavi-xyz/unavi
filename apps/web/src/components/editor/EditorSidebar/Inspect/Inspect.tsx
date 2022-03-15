import { useStore } from "../../../../helpers/editor/store";

import NumberField from "./fields/NumberField";
import TripletField from "./fields/TripletField";

export default function Inspect() {
  const selected = useStore((state) => state.selected);
  const name = useStore((state) => state.scene[selected?.id]?.name);
  const params = useStore((state) => state.scene[selected?.id]?.params);

  function handleChange(value: any, key: string) {
    const changes = { [key]: value };
    useStore.getState().updateInstanceParams(selected.id, changes);
  }

  function getHandleChange(key: string) {
    return (value: any) => handleChange(value, key);
  }

  if (!selected || !params) return null;

  return (
    <div className="space-y-6">
      <div className="text-3xl flex justify-center">{name}</div>

      <div className="space-y-1">
        <TripletField
          title="Position"
          value={params.position}
          onChange={getHandleChange("position")}
        />
        <TripletField
          title="Rotation"
          radians
          step={1}
          value={params.rotation}
          onChange={getHandleChange("rotation")}
        />

        {params?.scale && (
          <TripletField
            title="Scale"
            value={params.scale}
            onChange={getHandleChange("scale")}
          />
        )}

        {params?.radius && (
          <div className="py-4 space-y-1">
            <div className="text-xl text-neutral-500 mb-2">Geometry</div>

            {params?.radius && (
              <NumberField
                title="Radius"
                value={params?.radius}
                onChange={getHandleChange("radius")}
              />
            )}
          </div>
        )}
      </div>
    </div>
  );
}
