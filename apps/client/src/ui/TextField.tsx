import React from "react";

interface Props extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
}

const TextField = React.forwardRef<HTMLInputElement, Props>(
  ({ label, className, ...rest }, ref) => {
    return (
      <label className="block space-y-1">
        {label && <div className="font-bold text-neutral-700">{label}</div>}

        <input
          ref={ref}
          type="text"
          className={`w-full rounded-lg border border-neutral-200 px-3 py-2 ${className}`}
          {...rest}
        />
      </label>
    );
  }
);

TextField.displayName = "TextField";

export default TextField;
