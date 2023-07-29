import { forwardRef, InputHTMLAttributes } from "react";

interface Props extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  padding?: "normal" | "thin";
}

const TextFieldDark = forwardRef<HTMLInputElement, Props>(
  (
    { label, className, disabled, padding = "normal", children, ...rest },
    ref,
  ) => {
    return (
      <label className="block space-y-1">
        {label && <div className="font-bold text-neutral-400">{label}</div>}

        <div className="relative">
          <input
            ref={ref}
            type="text"
            disabled={disabled}
            className={`w-full rounded-lg bg-neutral-800 text-white placeholder:text-neutral-400 ${disabled ? "text-opacity-50" : ""
              } ${padding === "normal" ? "px-3 py-2" : "px-2 py-1"} ${className}`}
            {...rest}
          />

          {children}
        </div>
      </label>
    );
  },
);

TextFieldDark.displayName = "TextFieldDark";

export default TextFieldDark;
