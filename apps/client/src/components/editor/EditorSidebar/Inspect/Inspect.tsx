import { useStore } from "../../helpers/store";

import TransformModule from "./modules/TransformModule";
import GeometryModule from "./modules/GeometryModule";
import MaterialModule from "./modules/MaterialModule/MaterialModule";
import HeightmapModule from "./modules/HeightmapModule";

export default function Inspect() {
  const selected = useStore((state) => state.selected);
  const instance = useStore((state) => state.scene.instances[selected?.id]);

  if (!instance) return null;

  return (
    <div className="space-y-6">
      <div className="text-2xl flex items-center justify-center h-9">
        {instance.type}
      </div>

      <div className="border-t">
        <TransformModule />
        <GeometryModule />
        <MaterialModule />
        <HeightmapModule />
      </div>
    </div>
  );
}
