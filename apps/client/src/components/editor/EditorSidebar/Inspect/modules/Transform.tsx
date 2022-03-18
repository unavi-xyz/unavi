import { useStore } from "../../../helpers/store";
import { getHandleChange } from "../helpers";

import TripletField from "../inputs/TripletField";

export default function Transform() {
  const selected = useStore((state) => state.selected);
  const params = useStore(
    (state) => state.scene.instances[selected?.id]?.params
  );

  if (!params) return null;

  return (
    <div className="space-y-1">
      <div className="text-xl text-neutral-500 mb-2">Transform</div>

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
    </div>
  );
}
