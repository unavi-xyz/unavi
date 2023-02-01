import Avatar from "boring-avatars";
import Image from "next/image";

import { isFromCDN } from "../utils/isFromCDN";

interface Props {
  src?: string | null;
  circle?: boolean;
  draggable?: boolean;
  uniqueKey: string;
  size: number;
}

export default function ProfilePicture({ src, circle, draggable = true, uniqueKey, size }: Props) {
  const circleClass = circle ? "rounded-full" : "rounded-lg";

  if (!src) {
    if (uniqueKey)
      return (
        <Avatar
          size={size}
          name={uniqueKey}
          square={!circle}
          variant="beam"
          colors={["#38bdf8", "#FCECC9", "#FCB0B3", "#F93943", "#445E93"]}
        />
      );
    else return null;
  }

  return isFromCDN(src) ? (
    <Image
      src={src}
      width={size}
      height={size}
      draggable={draggable}
      alt=""
      className={`${circleClass} bg-neutral-200`}
    />
  ) : (
    <img
      src={src}
      draggable={draggable}
      width={size}
      height={size}
      alt=""
      className={`${circleClass} bg-neutral-200`}
      crossOrigin="anonymous"
    />
  );
}
