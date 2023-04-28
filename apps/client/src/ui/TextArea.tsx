import React from "react";

interface Props extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
}

const TextArea = React.forwardRef<HTMLTextAreaElement, Props>(({ label, ...rest }, ref) => {
  return (
    <label className="block space-y-1">
      {label && <div className="font-bold text-neutral-700">{label}</div>}

      <textarea
        ref={ref}
        className="max-h-64 w-full rounded-lg border border-neutral-200 p-4 py-2"
        {...rest}
      />
    </label>
  );
});

TextArea.displayName = "TextArea";

export default TextArea;
