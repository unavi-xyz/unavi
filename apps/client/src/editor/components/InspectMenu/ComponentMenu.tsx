import { MdClose } from "react-icons/md";

import { useEditorStore } from "@/app/editor/[id]/store";

interface Props {
  title?: string;
  removeable?: boolean;
  onRemove?: () => void;
  children?: React.ReactNode;
}

export default function ComponentMenu({ title, removeable = true, onRemove, children }: Props) {
  const isPlaying = useEditorStore((state) => state.isPlaying);

  const outlineClass = !isPlaying && removeable ? "hover:ring-1" : "";

  return (
    <div className={`group rounded-xl px-4 py-3 ring-neutral-300 transition ${outlineClass}`}>
      <div className="flex justify-between text-xl font-bold">
        {title && <div>{title}</div>}

        {removeable && (
          <button
            onClick={() => {
              if (isPlaying) return;
              if (onRemove) onRemove();
            }}
            disabled={isPlaying}
            className={`text-neutral-500 opacity-0 transition hover:text-black ${
              isPlaying ? "" : "focus:opacity-100 group-hover:opacity-100"
            }`}
          >
            <MdClose />
          </button>
        )}
      </div>

      {children && <div className="space-y-2 pt-2">{children}</div>}
    </div>
  );
}
