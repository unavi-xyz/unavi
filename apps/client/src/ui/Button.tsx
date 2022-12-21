import Spinner from "./Spinner";

export type ButtonVariant = "elevated" | "filled" | "tonal" | "outlined" | "text";

interface Props extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  color?: "primary" | "error";
  loading?: boolean;
  disabled?: boolean;
  fullWidth?: boolean;
  icon?: boolean;
  rounded?: "full" | "large" | "small";
  cursor?: "pointer" | "default";
  children: React.ReactNode;
}

export default function Button({
  variant = "text",
  color = "primary",
  loading = false,
  disabled = false,
  fullWidth = false,
  icon = false,
  rounded = "full",
  cursor = "pointer",
  children,
  ...rest
}: Props) {
  const textClass =
    variant === "text" ? (color === "primary" ? "hover:bg-sky-200" : "hover:bg-red-200") : null;

  const outlineClass =
    variant === "outlined"
      ? color === "primary"
        ? `ring-1 ring-neutral-500 hover:bg-sky-200`
        : `ring-1 ring-neutral-500 hover:text-red-900 hover:bg-red-200`
      : null;

  const tonalClass =
    variant === "tonal" ? (color === "primary" ? "bg-sky-200" : "bg-red-200 text-red-900") : null;

  const filledClass =
    variant === "filled" ? (color === "primary" ? "bg-sky-300" : "bg-red-700 text-white") : null;

  const elevatedClass =
    variant === "elevated" ? "shadow-dark bg-white text-sky-300 hover:bg-neutral-200/50" : null;

  const disabledClass =
    loading || disabled
      ? "opacity-40"
      : variant === "text"
      ? "active:bg-opacity-75"
      : variant === "outlined"
      ? "active:bg-opacity-75"
      : variant === "filled"
      ? "hover:shadow-dark hover:bg-opacity-90 active:bg-opacity-75"
      : variant === "tonal"
      ? "hover:shadow-dark hover:bg-opacity-90 active:bg-opacity-75"
      : null;

  const loadingClass = loading ? "opacity-0" : null;
  const fullWidthClass = fullWidth ? "w-full" : null;
  const roundClass =
    rounded === "small" ? "rounded-lg" : rounded === "large" ? "rounded-xl" : "rounded-full";

  const paddingClass = icon ? "p-2.5" : "px-5 py-1.5";
  const cursorClass =
    disabled || loading
      ? "cursor-not-allowed"
      : cursor === "pointer"
      ? "cursor-pointer"
      : "cursor-default";

  return (
    <button
      className={`relative flex select-none items-center justify-center font-bold ring-inset transition ${cursorClass} ${paddingClass} ${disabledClass} ${fullWidthClass} ${roundClass} ${textClass} ${outlineClass} ${tonalClass} ${filledClass} ${elevatedClass}`}
      {...rest}
    >
      <div className={`w-full ${loadingClass}`}>{children}</div>
      {loading && (
        <div className="absolute">
          <Spinner />
        </div>
      )}
    </button>
  );
}
