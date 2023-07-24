import { forwardRef } from "react";

type Props = React.ButtonHTMLAttributes<HTMLButtonElement>;

const Button = forwardRef<HTMLButtonElement, Props>(
  ({ className, disabled, children, ...rest }, ref) => {
    return (
      <button
        ref={ref}
        disabled={disabled}
        className={`rounded-full bg-neutral-900 px-6 py-1.5 font-bold text-white outline-2 outline-offset-4 transition ${
          disabled
            ? "cursor-default opacity-40"
            : "hover:opacity-90 hover:shadow active:opacity-80"
        } ${className}`}
        {...rest}
      >
        {children}
      </button>
    );
  }
);

Button.displayName = "Button";

export default Button;
