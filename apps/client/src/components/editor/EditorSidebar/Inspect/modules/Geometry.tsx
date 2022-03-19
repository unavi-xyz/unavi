import { useStore } from "../../../helpers/store";
import { getHandleChange, useSections } from "../helpers";

import NumberField from "../inputs/NumberField";

export default function Geometry() {
  const selected = useStore((state) => state.selected);
  const name = useStore((state) => state.scene.instances[selected?.id]?.name);
  const params = useStore(
    (state) => state.scene.instances[selected?.id]?.params
  );

  const sections = useSections(name);

  if (!params || !sections.includes("radius")) return null;

  return (
    <div className="space-y-1">
      <div className="text-xl text-neutral-500 mb-2">Geometry</div>

      {sections.includes("radius") && (
        <NumberField
          title="Radius"
          value={params.radius}
          onChange={getHandleChange("radius")}
        />
      )}
    </div>
  );
}
