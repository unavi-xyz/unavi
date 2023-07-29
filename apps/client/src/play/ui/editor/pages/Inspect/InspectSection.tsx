import { useState } from "react";
import { IoMdTrash } from "react-icons/io";

import Tooltip from "@/src/ui/Tooltip";

interface Props {
  title: string;
  onRemove?: () => void;
  children: React.ReactNode;
}

export default function InspectSection({ title, onRemove, children }: Props) {
  const [showRing, setShowRing] = useState(false);

  return (
    <div
      className={`group space-y-2 rounded-md ring-red-500 ring-offset-4 ring-offset-neutral-900 transition ${showRing ? "opacity-70 ring-2" : ""
        }`}
    >
      <div className="relative">
        <div className="text-center font-bold text-neutral-400">{title}</div>

        {onRemove && (
          <Tooltip text="Remove" side="top">
            <button
              onClick={onRemove}
              onMouseEnter={() => setShowRing(true)}
              onMouseLeave={() => setShowRing(false)}
              onFocus={() => setShowRing(true)}
              onBlur={() => setShowRing(false)}
              className="absolute inset-y-0 right-0 hidden p-0.5 text-neutral-500 hover:text-neutral-300 active:opacity-80 group-hover:block"
            >
              <IoMdTrash />
            </button>
          </Tooltip>
        )}
      </div>

      <div className="space-y-1">{children}</div>
    </div>
  );
}
