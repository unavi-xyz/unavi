import Image from "next/future/image";

interface Props {
  src?: string;
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

  return (
    <Image
      src={src}
      width={256}
      height={256}
      draggable={draggable}
      alt="profile picture"
      className={`${circleClass} bg-primaryContainer`}
    />
  );
}
