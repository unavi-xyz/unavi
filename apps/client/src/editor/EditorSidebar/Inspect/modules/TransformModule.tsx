import { useStore } from "../../../helpers/store";
import { getHandleChange } from "../helpers";

import TripletField from "../inputs/TripletField";
import Module from "./Module";

export default function TransformModule() {
  const selected = useStore((state) => state.selected);
  const properties = useStore(
    (state) => state.scene.instances[selected?.id]?.properties
  );

  if (!properties) return null;

  return (
    <Module title="Transform">
      <div className="space-y-1">
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
    </Module>
  );
}
