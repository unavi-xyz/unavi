import Spinner from "./Spinner";

export type ButtonVariant =
  | "elevated"
  | "filled"
  | "tonal"
  | "outlined"
  | "text";

interface Props extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  color?: "Primary" | "Tertiary";
  loading?: boolean;
  disabled?: boolean;
  fullWidth?: boolean;
  squared?: boolean;
  children: React.ReactNode;
}

export default function Button({
  variant = "text",
  color = "Primary",
  loading = false,
  disabled = false,
  fullWidth = false,
  squared = false,
  children,
  ...rest
}: Props) {
  const textClass =
    variant === "text"
      ? color === "Primary"
        ? "text-primary hover:bg-primaryContainer"
        : "text-tertiary hover:bg-tertiaryContainer"
      : "";
  const outlineClass =
    variant === "outlined"
      ? color === "Primary"
        ? "ring-1 ring-outline text-onPrimaryContainer hover:bg-primaryContainer"
        : "ring-1 ring-outline text-onTertiaryContainer hover:bg-tertiaryContainer"
      : null;
  const tonalClass =
    variant === "tonal"
      ? color === "Primary"
        ? "bg-primaryContainer text-onPrimaryContainer"
        : "bg-tertiaryContainer text-onTertiaryContainer"
      : null;
  const filledClass =
    variant === "filled"
      ? color === "Primary"
        ? "bg-primary text-onPrimary"
        : "bg-tertiary text-onTertiary"
      : null;
  const elevatedClass =
    variant === "elevated"
      ? color === "Primary"
        ? "shadow-dark bg-surface text-primary hover:bg-surfaceVariant/50"
        : "shadow-dark bg-surface text-tertiary hover:bg-surfaceVariant/50"
      : null;

  const disabledClass =
    loading || disabled
      ? "opacity-50 cursor-not-allowed"
      : variant === "tonal" || variant === "filled"
      ? "hover:shadow-dark"
      : null;

  const loadingClass = loading ? "opacity-0" : null;
  const fullWidthClass = fullWidth ? "w-full" : null;
  const squaredClass = squared ? "rounded-xl" : "rounded-full";

  return (
    <button
      className={`relative flex items-center justify-center transition font-bold
                  px-5 py-1.5 ${disabledClass} ${fullWidthClass} ${squaredClass}
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
