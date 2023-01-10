import ObjectsButton from "./ObjectsButton";
import SpecialsButton from "./SpecialsButton";
import TreeMenuRoot from "./TreeMenuRoot";

export default function TreeMenu() {
  return (
    <div className="pb-28">
      <div className="flex h-12 items-center justify-center space-x-2 py-1">
        <ObjectsButton />
        <SpecialsButton />
      </div>

      <div className="h-full w-full overflow-y-scroll">
        <TreeMenuRoot />
      </div>
    </div>
  );
}
