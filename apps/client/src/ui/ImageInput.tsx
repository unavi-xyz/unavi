import BoringAvatar from "boring-avatars";

import { BORING_AVATAR_COLORS } from "./Avatar";

interface Props extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "children"> {
  fallbackKey?: string;
  fallbackSize?: number;
}

export default function ImageInput({
  src,
  disabled,
  fallbackKey,
  fallbackSize,
  className = "w-full h-full",
  ...rest
}: Props) {
  return (
    <div className="relative">
      <label>
        <div
          className={`absolute cursor-pointer rounded-2xl transition ${
            disabled ? "" : "hover:bg-black/30 active:bg-black/20"
          } ${className}`}
        />

        <input disabled={disabled} type="file" accept="image/*" className="hidden" {...rest} />
      </label>

      {src ? (
        <img
          src={src}
          alt=""
          crossOrigin="anonymous"
          className={`rounded-2xl transition ${
            disabled ? "cursor-default opacity-70" : ""
          } ${className}`}
        />
      ) : fallbackKey ? (
        <BoringAvatar
          size={fallbackSize}
          name={fallbackKey}
          variant="beam"
          colors={BORING_AVATAR_COLORS}
        />
      ) : (
        <div
          className={`rounded-2xl bg-neutral-200 transition ${
            disabled ? "cursor-default opacity-70" : ""
          } ${className}`}
        />
      )}
    </div>
  );
}
