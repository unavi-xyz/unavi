import { useId } from "react";

type Props = Omit<React.InputHTMLAttributes<HTMLInputElement>, "children">;

export default function ImageInput({ src, disabled, className = "w-full h-full", ...rest }: Props) {
  const id = useId();

  return (
    <div className="relative">
      <label htmlFor={id}>
        <div
          className={`absolute cursor-pointer rounded-lg transition ${
            disabled ? "" : "hover:bg-black/30 active:bg-black/20"
          } ${className}`}
        />
      </label>

      {src ? (
        <img
          src={src}
          alt=""
          crossOrigin="anonymous"
          className={`rounded-lg transition ${
            disabled ? "cursor-not-allowed opacity-70" : ""
          } ${className}`}
        />
      ) : (
        <div
          className={`rounded-lg bg-neutral-200 transition ${
            disabled ? "cursor-not-allowed opacity-70" : ""
          } ${className}`}
        />
      )}

      <input id={id} disabled={disabled} type="file" className="hidden" {...rest} />
    </div>
  );
}
