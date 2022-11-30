import Avatar from "boring-avatars";
import Image from "next/image";

import { isFromCDN } from "../../utils/isFromCDN";

interface Props {
  src?: string | null;
  circle?: boolean;
  draggable?: boolean;
  uniqueKey: string;
  size: number;
}

export default function ProfilePicture({
  src,
  circle,
  draggable = true,
  uniqueKey,
  size,
}: Props) {
  const circleClass = circle ? "rounded-full" : "rounded-xl";

  if (!src) {
    if (uniqueKey)
      return (
        <Avatar
          size={size}
          name={uniqueKey}
          square={!circle}
          variant="beam"
          colors={["#52DAFF", "#FCECC9", "#FCB0B3", "#F93943", "#445E93"]}
        />
      );
    else return null;
  }

  return isFromCDN(src) ? (
    <Image
      src={src}
      width={256}
      height={256}
      draggable={draggable}
      alt=""
      className={`${circleClass} bg-primaryContainer`}
    />
  ) : (
    <img
      src={src}
      draggable={draggable}
      alt=""
      className={`${circleClass} h-full w-full bg-primaryContainer`}
      crossOrigin="anonymous"
    />
  );
}
