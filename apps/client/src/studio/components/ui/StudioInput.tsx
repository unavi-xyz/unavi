import { forwardRef } from "react";

import { useStudio } from "../Studio";

const StudioInput = forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  ({ disabled, ...rest }, ref) => {
    const { mode } = useStudio();

    const isDisabled = disabled || mode === "play";

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

StudioInput.displayName = "StudioInput";

export default StudioInput;
