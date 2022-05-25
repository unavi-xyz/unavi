import { useStudioStore } from "../../../helpers/studio/store";
import ObjectButton from "./ObjectButton";
import TreeMenuItem from "./TreeMenuItem";

export default function TreeMenu() {
  const tree = useStudioStore((state) => state.scene.tree);

  return (
    <div className="h-full">
      <div className="py-2 h-14 flex items-center justify-center">
        <ObjectButton />
      </div>

      <div className="py-2 pr-4 w-full h-full">
        <TreeMenuItem entity={tree} isRoot />
      </div>
    </div>
  );
}
