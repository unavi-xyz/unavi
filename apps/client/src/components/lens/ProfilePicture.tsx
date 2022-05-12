import { useMediaImage } from "../../helpers/lens/hooks/useMediaImage";
import { Profile } from "../../generated/graphql";

interface Props {
  profile: Profile | undefined;
  circle?: boolean;
}

export default function ProfilePicture({ profile, circle }: Props) {
  const url = useMediaImage(profile?.picture);

  const circleClass = circle ? "rounded-full" : "rounded-xl";
  const identicon = `https://avatar.tobi.sh/${profile?.ownedBy}_${profile?.handle}.png`;

  return (
    <img
      src={url ?? identicon}
      alt="profile picture"
      className={`object-cover w-screen aspect-square ${circleClass}`}
    />
  );
}
