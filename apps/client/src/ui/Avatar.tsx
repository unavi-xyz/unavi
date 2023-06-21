import BoringAvatar from "boring-avatars";
import Image from "next/image";

import { isFromCDN } from "../utils/isFromCDN";

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
      className={`${
        loading ? "animate-pulse bg-neutral-300" : "bg-neutral-200"
      } ${circle ? "rounded-full" : "rounded-lg"}`}
      style={{ height: size, width: size }}
    >
      {loading ? null : src ? (
        isFromCDN(src) ? (
          <Image
            src={src}
            priority
            width={size}
            height={size}
            sizes={`${size}px`}
            alt=""
            draggable={draggable}
            className={`${circle ? "rounded-full" : "rounded-lg"}`}
          />
        ) : (
          <img
            src={src}
            sizes={`${size}px`}
            alt=""
            draggable={draggable}
            className={`${circle ? "rounded-full" : "rounded-lg"}`}
            style={{ height: size, width: size }}
            crossOrigin="anonymous"
          />
        )
      ) : (
        <BoringAvatar
          size={size}
          name={uniqueKey}
          square={!circle}
          variant="beam"
          colors={BORING_AVATAR_COLORS}
        />
      )}
    </div>
  );
}

export const BORING_AVATAR_COLORS = [
  "#38bdf8",
  "#FCECC9",
  "#FCB0B3",
  "#F93943",
  "#445E93",
];
