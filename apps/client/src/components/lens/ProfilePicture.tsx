import Image from "next/image";

import { Profile } from "../../generated/graphql";
import { useMediaImage } from "../../helpers/lens/hooks/useMediaImage";

interface Props {
  profile: Profile | undefined;
  circle?: boolean;
  draggable?: boolean;
}

export default function ProfilePicture({
  profile,
  circle,
  draggable = true,
}: Props) {
  const url = useMediaImage(profile?.picture);

  const circleClass = circle ? "rounded-full" : "rounded-xl";
  const identicon = `https://avatar.tobi.sh/${profile?.ownedBy}_${profile?.handle}.png`;

  return (
    <Image
      width="256px"
      height="256px"
      src={url ?? identicon}
      draggable={draggable}
      alt="profile picture"
      className={`${circleClass} bg-tertiaryContainer`}
    />
  );
}
