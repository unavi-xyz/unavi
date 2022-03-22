import { useStore } from "../../../helpers/store";
import { getHandleChange } from "../helpers";

import TripletField from "../inputs/TripletField";

export default function Transform() {
  const selected = useStore((state) => state.selected);
  const properties = useStore(
    (state) => state.scene.instances[selected?.id]?.properties
  );

  if (!properties) return null;

  return (
    <div className="space-y-1">
      <div className="text-xl text-neutral-500 mb-2">Transform</div>

      <TripletField
        title="Position"
        value={properties.position}
        onChange={getHandleChange("position")}
      />

      <TripletField
        title="Rotation"
        radians
        step={1}
        value={properties.rotation}
        onChange={getHandleChange("rotation")}
      />

      {"scale" in properties && (
        <TripletField
          title="Scale"
          value={properties.scale}
          onChange={getHandleChange("scale")}
        />
      )}
    </div>
  );
}
