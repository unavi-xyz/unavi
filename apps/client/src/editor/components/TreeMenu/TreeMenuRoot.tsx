import { useNodes } from "../../hooks/useNodes";
import { useEditorStore } from "../../store";
import TreeMenuItem from "./TreeMenuItem";

export default function TreeMenuRoot() {
  const engine = useEditorStore((state) => state.engine);

  const nodes = useNodes();
  const rootNodes = nodes.filter((node) => !engine?.modules.scene.node.getParent(node));
  const nodeIds = rootNodes.map((node) => engine?.modules.scene.node.getId(node));

  return (
    <div>
      {nodeIds?.map((id) => {
        if (!id) return null;
        return <TreeMenuItem key={id} id={id} />;
      })}
    </div>
  );
}
