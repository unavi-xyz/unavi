import React from "react";

import { useEditorStore } from "@/app/editor/[id]/store";

const EditorInput = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  ({ disabled, ...rest }, ref) => {
    const isPlaying = useEditorStore((state) => state.isPlaying);

    const isDisabled = disabled || isPlaying;

    return (
      <input
        ref={ref}
        disabled={isDisabled}
        className={`w-full rounded-md border border-neutral-200 pl-1.5 leading-snug ${
          isDisabled ? "" : "hover:bg-neutral-100 focus:bg-neutral-100"
        } `}
        {...rest}
      />
    );
  }
);

EditorInput.displayName = "EditorInput";

export default EditorInput;
