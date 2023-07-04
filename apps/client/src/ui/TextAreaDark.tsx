import { forwardRef } from "react";

interface Props extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
}

const TextAreaDark = forwardRef<HTMLTextAreaElement, Props>(
  ({ label, className, children, ...rest }, ref) => {
    return (
      <label className="block space-y-1">
        {label && <div className="font-bold text-neutral-400">{label}</div>}

        <div className="relative">
          <textarea
            ref={ref}
            className={`max-h-64 w-full rounded-lg bg-neutral-800 p-4 py-2 text-white placeholder:text-neutral-400 ${className}`}
            {...rest}
          />

          {children}
        </div>
      </label>
    );
  }
);

TextAreaDark.displayName = "TextAreaDark";

export default TextAreaDark;
