import { useContext } from "react";

import { HANDLE_ENDING, LensContext, useMediaImage } from "@wired-xr/lens";
import { useGetProfileQuery } from "@wired-xr/lens";

import ProfilePicture from "./ProfilePicture";

interface Props {
  circle?: boolean;
  draggable?: boolean;
}

export default function ViewerProfilePicture({ circle, draggable }: Props) {
  const { handle } = useContext(LensContext);

  const [{ data }] = useGetProfileQuery({
    variables: { request: { handles: [handle?.concat(HANDLE_ENDING)] } },
    pause: !handle,
  });

  const profile = data?.profiles.items[0];
  const src =
    useMediaImage(profile?.picture) ?? `https://avatar.tobi.sh/${handle}`;

  return <ProfilePicture circle={circle} draggable={draggable} src={src} />;
}
