import { useAtomValue } from "jotai";

import { selectedAtom } from "../../../helpers/studio/atoms";

import Transform from "./Transform";

export default function InspectMenu() {
  const selected = useAtomValue(selectedAtom);

  if (!selected) return null;

  return (
    <div className="p-4 space-y-4 w-full">
      <div className="flex justify-center text-xl font-bold">
        {selected.name}
      </div>

      <div>
        <Transform />
      </div>
    </div>
  );
}
