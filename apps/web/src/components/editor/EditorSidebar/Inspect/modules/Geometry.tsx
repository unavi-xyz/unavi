import { useStore } from "../../../../../helpers/editor/store";
import { getHandleChange } from "../helpers";

import NumberField from "../inputs/NumberField";

export default function Geometry() {
  const selected = useStore((state) => state.selected);
  const params = useStore(
    (state) => state.scene.instances[selected?.id]?.params
  );

  if (!params?.radius) return null;

  return (
    <div className="space-y-1">
      <div className="text-xl text-neutral-500 mb-2">Geometry</div>

      {params?.radius && (
        <NumberField
          title="Radius"
          value={params.radius}
          onChange={getHandleChange("radius")}
        />
      )}
    </div>
  );
}
