import ObjectsButton from "./ObjectsButton";
import SpecialsButton from "./SpecialsButton";
import TreeMenuRoot from "./TreeMenuRoot";

export default function TreeMenu() {
  return (
    <div className="h-full space-y-2 p-2 pr-0 pb-28">
      <div className="flex h-10 items-center justify-center space-x-2">
        <ObjectsButton />
        <SpecialsButton />
      </div>

      <div className="h-full w-full overflow-y-scroll">
        <TreeMenuRoot />
      </div>
    </div>
  );
}
