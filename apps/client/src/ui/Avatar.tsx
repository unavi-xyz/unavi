import BoringAvatar from "boring-avatars";

interface Props {
  src?: string | null;
  circle?: boolean;
  draggable?: boolean;
  loading?: boolean;
  uniqueKey?: string;
  size: number;
}

export default function Avatar({
  src,
  circle = false,
  draggable = false,
  loading = false,
  uniqueKey,
  size,
}: Props) {
  return (
    <div
      className={`${loading ? "animate-pulse bg-neutral-300" : "bg-neutral-200"} ${
        circle ? "rounded-full" : "rounded-lg"
      }`}
      style={{ width: size, height: size }}
    >
      {loading ? null : src ? (
        <img
          src={src}
          alt=""
          crossOrigin="anonymous"
          draggable={draggable}
          className={`${circle ? "rounded-full" : "rounded-lg"}`}
          style={{ width: size, height: size }}
        />
      ) : (
        <BoringAvatar
          size={size}
          name={uniqueKey}
          square={!circle}
          variant="beam"
          colors={["#38bdf8", "#FCECC9", "#FCB0B3", "#F93943", "#445E93"]}
        />
      )}
    </div>
  );
}
