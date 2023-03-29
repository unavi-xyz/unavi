import React from "react";

interface Props extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
}

const TextField = React.forwardRef<HTMLInputElement, Props>(
  ({ label, className, ...rest }, ref) => {
    return (
      <label className="block">
        {label && <div className="pb-1 text-lg font-bold">{label}</div>}

        <input
          ref={ref}
          type="text"
          className={`h-full w-full rounded-xl border border-neutral-300 px-3 py-2 text-neutral-900 placeholder:text-neutral-400 hover:border-neutral-400 ${className}`}
          {...rest}
        />
      </label>
    );
  }
);

TextField.displayName = "TextField";

export default TextField;
