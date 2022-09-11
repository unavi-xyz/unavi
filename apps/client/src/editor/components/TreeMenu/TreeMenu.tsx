import ObjectsButton from "./ObjectsButton";
import TreeMenuRoot from "./TreeMenuRoot";

export default function TreeMenu() {
  return (
    <div className="h-full p-2 pl-3 space-y-2">
      <div className="h-10 flex items-center justify-center space-x-2">
        <ObjectsButton />
      </div>

      <div className="w-full h-full overflow-y-auto overflow-x-hidden">
        <TreeMenuRoot />
      </div>
    </div>
  );
}
