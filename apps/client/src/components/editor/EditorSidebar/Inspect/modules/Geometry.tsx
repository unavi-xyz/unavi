import { useStore } from "../../../helpers/store";
import { getHandleChange } from "../helpers";

import NumberField from "../inputs/NumberField";

export default function Geometry() {
  const selected = useStore((state) => state.selected);
  const type = useStore((state) => state.scene.instances[selected?.id]?.type);
  const properties = useStore(
    (state) => state.scene.instances[selected?.id]?.properties
  );

  if (!properties || !("radius" in properties)) return null;

  return (
    <div className="space-y-1">
      <div className="text-xl text-neutral-500 mb-2">Geometry</div>

      {"radius" in properties && (
        <NumberField
          title="Radius"
          value={properties.radius}
          onChange={getHandleChange("radius")}
        />
      )}
    </div>
  );
}
