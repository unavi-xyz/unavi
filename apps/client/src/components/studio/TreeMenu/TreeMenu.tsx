import { useStudioStore } from "../../../helpers/studio/store";

import ObjectButton from "./ObjectButton";
import TreeMenuItem from "./TreeMenuItem";

export default function TreeMenu() {
  const children = useStudioStore((state) => state.scene.tree.children);

  return (
    <div className="h-full">
      <div className="py-2 h-14 flex items-center justify-center border-b">
        <ObjectButton />
      </div>
      <div
        onClick={() => useStudioStore.setState({ selectedId: undefined })}
        className="p-4 space-y-4 w-full h-full"
      >
        <div>
          {children.map((object) => (
            <TreeMenuItem key={object.id} object={object} />
          ))}
        </div>
      </div>
    </div>
  );
}
