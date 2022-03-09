import { useAtom } from "jotai";

import { selectedAtom } from "../../../../helpers/editor/state";
import TripletField from "./fields/TripletField";

export default function Inspect() {
  const [selected, setSelected] = useAtom(selectedAtom);

  const params = selected.instance.params;

  function handleChange(value: any, key: string) {
    setSelected((prev) => {
      const newValue = { ...prev };
      newValue.instance.params[key] = value;
      return newValue;
    });
  }

  function getHandleChange(key: string) {
    return (value: any) => handleChange(value, key);
  }

  if (!selected) return null;

  return (
    <div className="space-y-6">
      <div className="text-3xl flex justify-center">
        {selected.instance.asset}
      </div>

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
        <TripletField
          title="Scale"
          value={params.scale}
          onChange={getHandleChange("scale")}
        />
      </div>
    </div>
  );
}
