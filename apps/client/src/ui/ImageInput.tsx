type Props = Omit<React.InputHTMLAttributes<HTMLInputElement>, "children">;

export default function ImageInput({
  src,
  disabled,
  name,
  className = "w-full h-full",
  ...rest
}: Props) {
  return (
    <div className="relative">
      <label>
        {name && <div className="pb-1 text-lg font-bold">{name}</div>}

        <div
          className={`absolute cursor-pointer rounded-2xl transition ${
            disabled ? "" : "hover:bg-black/30 active:bg-black/20"
          } ${className}`}
        />

        <input
          name={name}
          disabled={disabled}
          type="file"
          accept="image/*"
          className="hidden"
          {...rest}
        />
      </label>

      {src ? (
        <img
          src={src}
          alt=""
          crossOrigin="anonymous"
          className={`rounded-2xl transition ${
            disabled ? "cursor-not-allowed opacity-70" : ""
          } ${className}`}
        />
      ) : (
        <div
          className={`rounded-2xl bg-neutral-200 transition ${
            disabled ? "cursor-not-allowed opacity-70" : ""
          } ${className}`}
        />
      )}
    </div>
  );
}
