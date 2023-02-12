import { NodeSpecJSON } from "@behave-graph/core";

interface Props {
  title: string;
  category?: NodeSpecJSON["category"];
  selected: boolean;
  children: React.ReactNode;
}

export default function NodeContainer({ title, category, selected, children }: Props) {
  return (
    <div
      className={`min-w-[120px] rounded bg-white/80 text-sm text-neutral-900 shadow hover:shadow-md
      ${selected ? "outline outline-2 outline-neutral-900" : ""}`}
    >
      <div
        className={`rounded-t bg-gradient-to-t from-black/10 to-white/10 px-2 leading-normal text-white ${
          category === "Action"
            ? "bg-sky-500/80"
            : category === "Query"
            ? "bg-purple-500/80"
            : category === "Logic"
            ? "bg-green-500/80"
            : category === "Event"
            ? "bg-red-500/80"
            : category === "Variable"
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
