import ObjectsButton from "./ObjectsButton";
import SpecialsButton from "./SpecialsButton";
import TreeRoot from "./TreeRoot";

export default function TreeMenu() {
  return (
    <div className="h-full space-y-1 pb-12">
      <div className="flex h-12 items-center justify-center space-x-2 py-1">
        <ObjectsButton />
        <SpecialsButton />
      </div>

      <div className="h-full w-full overflow-y-auto overflow-x-hidden">
        <TreeRoot />
      </div>
    </div>
  );
}
