import React from "react";

interface Props extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
}

const TextArea = React.forwardRef<HTMLTextAreaElement, Props>(
  ({ label, disabled, ...rest }, ref) => {
    return (
      <label className="block">
        {label && <div className="pb-1 text-lg font-bold">{label}</div>}

        <textarea
          ref={ref}
          disabled={disabled}
          className={`h-full max-h-64 w-full rounded-xl border border-neutral-300 px-3 py-2 text-neutral-900 placeholder:text-neutral-400 ${
            disabled ? "" : "hover:border-neutral-400"
          }`}
          {...rest}
        />
      </label>
    );
  }
);

TextArea.displayName = "TextArea";

export default TextArea;
