import { forwardRef, InputHTMLAttributes } from "react";

interface Props extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
}

const TextField = forwardRef<HTMLInputElement, Props>(
  ({ label, className, children, ...rest }, ref) => {
    return (
      <label className="block space-y-1">
        {label && <div className="font-bold text-neutral-700">{label}</div>}

        <div className="relative">
          <input
            ref={ref}
            type="text"
            className={`w-full rounded-lg border border-neutral-300 px-3 py-2 placeholder:text-neutral-400 ${className}`}
            {...rest}
          />

          {children}
        </div>
      </label>
    );
  }
);

TextField.displayName = "TextField";

export default TextField;
