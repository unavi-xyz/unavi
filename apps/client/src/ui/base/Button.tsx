import Spinner from "./Spinner";

export type ButtonVariant = "elevated" | "filled" | "tonal" | "outlined" | "text";

interface Props extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  color?: "primary" | "primary";
  loading?: boolean;
  disabled?: boolean;
  fullWidth?: boolean;
  icon?: boolean;
  squared?: "large" | "small" | undefined;
  children: React.ReactNode;
}

export default function Button({
  variant = "text",
  color = "primary",
  loading = false,
  disabled = false,
  fullWidth = false,
  icon = false,
  squared,
  children,
  ...rest
}: Props) {
  const textClass =
    variant === "text"
      ? color === "primary"
        ? "hover:bg-primaryContainer hover:text-onPrimaryContainer"
        : "hover:bg-primaryContainer hover:text-onPrimaryContainer"
      : "";

  const outlineClass =
    variant === "outlined"
      ? color === "primary"
        ? `ring-1 ring-outline hover:ring-onPrimaryContainer
           hover:text-onPrimaryContainer hover:bg-primaryContainer`
        : `ring-1 ring-outline hover:ring-onPrimaryContainer
           hover:text-onPrimaryContainer hover:bg-primaryContainer`
      : null;

  const tonalClass = variant === "tonal" ? "bg-primaryContainer text-onPrimaryContainer" : null;

  const filledClass =
    variant === "filled"
      ? color === "primary"
        ? "bg-primary text-onPrimary"
        : "bg-primary text-onPrimary"
      : null;

  const elevatedClass =
    variant === "elevated"
      ? "shadow-dark bg-surface text-primary hover:bg-surfaceVariant/50"
      : null;

  const disabledClass =
    loading || disabled
      ? "opacity-40 cursor-not-allowed bg-surfaceVariant"
      : variant === "text"
      ? "active:bg-opacity-75"
      : variant === "outlined"
      ? "active:bg-opacity-75"
      : variant === "filled"
      ? "hover:shadow-filled hover:bg-opacity-90 active:bg-opacity-75"
      : variant === "tonal"
      ? "hover:shadow-tonal hover:bg-opacity-90 active:bg-opacity-75"
      : null;

  const loadingClass = loading ? "opacity-0" : null;
  const fullWidthClass = fullWidth ? "w-full" : null;
  const squaredClass =
    squared === "small" ? "rounded" : squared === "large" ? "rounded-xl" : "rounded-full";

  const paddingClass = icon ? "p-2.5" : "px-5 py-1.5";

  return (
    <button
      className={`relative flex items-center justify-center transition font-bold
                  ${paddingClass} ${disabledClass} ${fullWidthClass} ${squaredClass}
                  ${textClass} ${outlineClass} ${tonalClass} ${filledClass} ${elevatedClass}`}
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
