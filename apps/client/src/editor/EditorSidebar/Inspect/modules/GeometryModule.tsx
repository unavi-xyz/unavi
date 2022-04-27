import { useStore } from "../../../helpers/store";
import { getHandleChange } from "../helpers";

import NumberField from "../inputs/NumberField";
import Module from "./Module";

export default function GeometryModule() {
  const selected = useStore((state) => state.selected);
  const properties = useStore(
    (state) => state.scene.instances[selected?.id]?.properties
  );

  if (!properties || !("radius" in properties)) return null;

  return (
    <Module title="Geometry">
      <div className="space-y-1">
        {"radius" in properties && (
          <NumberField
            title="Radius"
            value={properties.radius}
            onChange={getHandleChange("radius")}
          />
        )}
      </div>
    </Module>
  );
}
