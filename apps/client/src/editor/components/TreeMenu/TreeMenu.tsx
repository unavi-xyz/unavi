import ObjectsButton from "./ObjectsButton";
import TreeMenuRoot from "./TreeMenuRoot";

export default function TreeMenu() {
  return (
    <div className="h-full flex flex-col">
      <div className="py-2 h-14 flex items-center justify-center space-x-2">
        <ObjectsButton />
      </div>

      <div className="py-2 pl-4 pr-2 w-full h-full overflow-y-auto overflow-x-hidden">
        <TreeMenuRoot />
      </div>
    </div>
  );
}
