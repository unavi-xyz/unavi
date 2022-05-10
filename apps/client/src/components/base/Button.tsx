import Spinner from "./Spinner";

interface Props {
  color?: "black" | "gray" | "red";
  loading?: boolean;
  disabled?: boolean;
  outline?: boolean;
  fullWidth?: boolean;
  children: React.ReactNode;
  [key: string]: any;
}

export default function Button({
  color = "black",
  loading = false,
  disabled = false,
  outline = false,
  fullWidth = false,
  children,
  ...rest
}: Props) {
  const cursorClass =
    disabled || loading ? "cursor-not-allowed" : "cursor-pointer";
  const outlineClass = outline ? "ring-1" : "";

  const ringColorClass =
    color === "black"
      ? "ring-black"
      : color === "gray"
      ? "ring-neutral-500"
      : "ring-red-500";

  const bgColorClass = outline
    ? "bg-transparent"
    : disabled || loading
    ? "bg-neutral-500"
    : color === "black"
    ? "bg-black"
    : color === "gray"
    ? "bg-neutral-500"
    : "bg-red-500";

  const textColorClass = outline
    ? color === "black"
      ? "text-black"
      : color === "gray"
      ? "text-neutral-500"
      : "text-red-500"
    : "text-white";

  const hoverColorClass = outline
    ? color === "black"
      ? "hover:bg-neutral-200"
      : color === "gray"
      ? "hover:bg-neutral-200"
      : "hover:bg-red-100"
    : null;

  const widthClass = fullWidth ? "w-full" : "";

  return (
    <div
      className={`px-6 h-9 rounded-full flex items-center justify-center ${widthClass}
                  transition-all duration-150 font-bold ${textColorClass} ${hoverColorClass}
                  ${bgColorClass} ${cursorClass} ${outlineClass} ${ringColorClass}`}
      {...rest}
    >
      {loading ? <Spinner /> : children}
    </div>
  );
}
