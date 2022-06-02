import { useStudioStore } from "../../../helpers/studio/store";
import LightsButton from "./LightsButton";
import ObjectsButton from "./ObjectsButton";
import TreeMenuItem from "./TreeMenuItem";

export default function TreeMenu() {
  const tree = useStudioStore((state) => state.scene.tree);

  return (
    <div className="h-full flex flex-col">
      <div className="py-2 h-14 flex items-center justify-center space-x-2">
        <ObjectsButton />
        <LightsButton />
      </div>

      <div className="pt-2 pr-4 w-full h-full overflow-y-auto">
        <TreeMenuItem entity={tree} isRoot />
      </div>
    </div>
  );
}
