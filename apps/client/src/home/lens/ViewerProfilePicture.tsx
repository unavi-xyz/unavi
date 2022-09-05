import { useGetProfileQuery } from "@wired-labs/lens";

import { HANDLE_ENDING } from "../../lib/lens/constants";
import { useLens } from "../../lib/lens/hooks/useLens";
import { getMediaURL } from "../../lib/lens/utils/getMediaURL";
import ProfilePicture from "./ProfilePicture";

interface Props {
  circle?: boolean;
  draggable?: boolean;
}

export default function ViewerProfilePicture({ circle, draggable }: Props) {
  const { handle } = useLens();

  const [{ data }] = useGetProfileQuery({
    variables: { request: { handles: [handle?.concat(HANDLE_ENDING)] } },
    pause: !handle,
  });

  const profile = data?.profiles.items[0];
  const src =
    getMediaURL(profile?.picture) ?? `https://avatar.tobi.sh/${handle}`;

  return <ProfilePicture circle={circle} draggable={draggable} src={src} />;
}
