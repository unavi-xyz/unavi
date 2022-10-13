import Spinner from "./Spinner";

export type ButtonVariant =
  | "elevated"
  | "filled"
  | "tonal"
  | "outlined"
  | "text";

interface Props extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  color?: "primary" | "error";
  loading?: boolean;
  disabled?: boolean;
  fullWidth?: boolean;
  icon?: boolean;
  rounded?: "full" | "large" | "small";
  labelId?: string;
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
  labelId,
  children,
  ...rest
}: Props) {
  const textClass =
    variant === "text"
      ? color === "primary"
        ? "hover:bg-primaryContainer hover:text-onPrimaryContainer"
        : "hover:bg-errorContainer hover:text-onErrorContainer"
      : null;

  const outlineClass =
    variant === "outlined"
      ? color === "primary"
        ? `ring-1 ring-outline
           hover:text-onPrimaryContainer hover:bg-primaryContainer`
        : `ring-1 ring-outline
           hover:text-onErrorContainer hover:bg-errorContainer`
      : null;

  const tonalClass =
    variant === "tonal"
      ? color === "primary"
        ? "bg-primaryContainer text-onPrimaryContainer"
        : "bg-errorContainer text-onErrorContainer"
      : null;

  const filledClass =
    variant === "filled"
      ? color === "primary"
        ? "bg-primary text-onPrimary"
        : "bg-error text-onError"
      : null;

  const elevatedClass =
    variant === "elevated"
      ? "shadow-dark bg-surface text-primary hover:bg-surfaceVariant/50"
      : null;

  const disabledClass = loading
    ? "bg-opacity-40 cursor-not-allowed bg-surfaceVariant"
    : disabled
    ? "opacity-40 cursor-not-allowed bg-surfaceVariant"
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
    rounded === "small"
      ? "rounded-lg"
      : rounded === "large"
      ? "rounded-xl"
      : "rounded-full";

  const paddingClass = icon ? "p-2.5" : "px-5 py-1.5";

  if (labelId) {
    return (
      <label
        htmlFor={labelId}
        className={`relative flex cursor-pointer items-center justify-center text-center font-bold transition ${paddingClass} ${disabledClass} ${fullWidthClass} ${roundClass} ${textClass} ${outlineClass} ${tonalClass} ${filledClass} ${elevatedClass}`}
      >
        <div className={`w-full ${loadingClass}`}>{children}</div>
        {loading && (
          <div className="absolute">
            <Spinner />
          </div>
        )}
      </label>
    );
  }

  return (
    <button
      className={`relative flex items-center justify-center font-bold transition ${paddingClass} ${disabledClass} ${fullWidthClass} ${roundClass} ${textClass} ${outlineClass} ${tonalClass} ${filledClass} ${elevatedClass}`}
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
