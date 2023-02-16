import React, { ButtonHTMLAttributes } from "react";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  selected?: boolean;
  rounded?: "full" | "small";
  cursor?: "pointer" | "default";
}

const IconButton = React.forwardRef<HTMLButtonElement, Props>(
  ({ selected, rounded = "small", cursor = "default", children, ...rest }, ref) => {
    const selectedClass = selected ? "bg-neutral-200 hover:bg-neutral-300" : "hover:bg-neutral-200";
    const roundedClass = rounded === "full" ? "rounded-full" : "rounded-lg";
    const cursorClass = cursor === "pointer" ? "cursor-pointer" : "cursor-default";

    return (
      <button
        ref={ref}
        className={`flex aspect-square h-full items-center justify-center text-2xl transition active:opacity-80 ${cursorClass} ${roundedClass} ${selectedClass}`}
        {...rest}
      >
        {children}
      </button>
    );
  }
);

IconButton.displayName = "IconButton";

export default IconButton;
