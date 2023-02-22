import { NodeCategory, NodeSpecJSON } from "@wired-labs/behave-graph-core";

import { useEditorStore } from "../../store";

interface Props {
  id: string;
  title: string;
  category?: NodeSpecJSON["category"];
  selected: boolean;
  children: React.ReactNode;
}

export default function NodeContainer({ id, title, category, selected, children }: Props) {
  return (
    <div
      onContextMenu={() => useEditorStore.setState({ contextMenuNodeId: id })}
      className={`min-w-[120px] rounded bg-white/80 text-sm text-neutral-900 shadow hover:shadow-md
      ${selected ? "outline outline-2 outline-neutral-900" : ""}`}
    >
      <div
        className={`rounded-t bg-gradient-to-t from-black/10 to-white/10 px-2 capitalize leading-normal text-white ${
          category === NodeCategory.Action
            ? "bg-sky-500/80"
            : category === NodeCategory.Query
            ? "bg-purple-500/80"
            : category === NodeCategory.Logic
            ? "bg-green-500/80"
            : category === NodeCategory.Event
            ? "bg-red-500/80"
            : category === NodeCategory.Variable
            ? "bg-orange-500/80"
            : "bg-neutral-500/80"
        }`}
      >
        {title}
      </div>
      <div className="flex flex-col gap-2 py-2">{children}</div>
    </div>
  );
}
