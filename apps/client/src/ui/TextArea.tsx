import { forwardRef } from "react";

interface Props extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
}

const TextArea = forwardRef<HTMLTextAreaElement, Props>(
  ({ label, className, children, ...rest }, ref) => {
    return (
      <label className="block space-y-1">
        {label && <div className="font-bold text-neutral-700">{label}</div>}

        <div className="relative">
          <textarea
            ref={ref}
            className={`max-h-64 w-full rounded-lg border border-neutral-200 p-4 py-2 placeholder:text-neutral-400 ${className}`}
            {...rest}
          />

          {children}
        </div>
      </label>
    );
  }
);

TextArea.displayName = "TextArea";

export default TextArea;
