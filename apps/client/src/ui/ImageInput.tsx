import { useId } from "react";

type Props = React.InputHTMLAttributes<HTMLInputElement>;

export default function ImageInput({ src, className, ...rest }: Props) {
  const id = useId();

  return (
    <div className="relative">
      <label htmlFor={id}>
        <div
          className={`absolute cursor-pointer transition hover:bg-black/30 active:bg-black/20 ${className}`}
        />
      </label>

      {src ? (
        <img src={src} alt="" crossOrigin="anonymous" className={className} />
      ) : (
        <div className={`bg-neutral-200 ${className}`} />
      )}

      <input id={id} type="file" className="hidden" {...rest} />
    </div>
  );
}
