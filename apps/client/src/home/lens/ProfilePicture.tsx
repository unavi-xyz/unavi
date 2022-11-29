import Image from "next/image";

import { isFromCDN } from "../../utils/isFromCDN";

interface Props {
  src?: string | null;
  circle?: boolean;
  draggable?: boolean;
}

export default function ProfilePicture({
  src,
  circle,
  draggable = true,
}: Props) {
  const circleClass = circle ? "rounded-full" : "rounded-xl";

  if (!src) return null;

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
