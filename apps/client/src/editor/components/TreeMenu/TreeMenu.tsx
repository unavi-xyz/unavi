import ObjectsButton from "./ObjectsButton";
import TreeMenuRoot from "./TreeMenuRoot";

export default function TreeMenu() {
  return (
    <div className="h-full space-y-2 p-2 pl-3">
      <div className="flex h-10 items-center justify-center space-x-2">
        <ObjectsButton />
      </div>

      <div className="h-full w-full overflow-y-auto overflow-x-hidden">
        <TreeMenuRoot />
      </div>
    </div>
  );
}
