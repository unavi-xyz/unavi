import Image from "next/image";

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
      width="256px"
      height="256px"
      src={src}
      draggable={draggable}
      alt="profile picture"
      className={`${circleClass} bg-primaryContainer`}
    />
  );
}
