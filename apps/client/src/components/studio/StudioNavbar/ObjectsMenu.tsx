import { Primitive, PRIMITIVES } from "scene";
import { useStudioStore } from "../../../helpers/studio/store";

export default function ObjectsMenu() {
  const selected = useStudioStore((state) => state.selected);
  const addPrimitive = useStudioStore((state) => state.addPrimitive);

  return (
    <div className="py-2 space-y-2">
      <div className="px-2 space-y-2">
        {Object.keys(PRIMITIVES).map((primitive) => (
          <button
            key={primitive}
            onClick={() => {
              const object = addPrimitive(primitive as Primitive, selected?.id);
              useStudioStore.setState({ selected: object });
            }}
            className="w-full flex cursor-pointer hover:bg-neutral-100 rounded-md px-3 py-1"
          >
            {primitive}
          </button>
        ))}
      </div>
    </div>
  );
}
