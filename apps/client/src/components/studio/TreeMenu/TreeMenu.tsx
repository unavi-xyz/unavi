import { useStudioStore } from "../../../helpers/studio/store";

import TreeMenuItem from "./TreeMenuItem";

export default function TreeMenu() {
  const children = useStudioStore((state) => state.tree.children);

  return (
    <div
      onClick={() => useStudioStore.setState({ selected: undefined })}
      className="p-4 space-y-4 w-full h-full"
    >
      <div>
        {children.map((object) => (
          <TreeMenuItem key={object.id} object={object} />
        ))}
      </div>
    </div>
  );
}
