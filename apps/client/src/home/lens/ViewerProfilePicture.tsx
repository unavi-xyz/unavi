import { useGetProfileQuery } from "@wired-labs/lens";
import { useEffect, useState } from "react";

import { HANDLE_ENDING } from "../../lib/lens/constants";
import { useLens } from "../../lib/lens/hooks/useLens";
import { getMediaURL } from "../../utils/getMediaURL";
import ProfilePicture from "./ProfilePicture";

interface Props {
  circle?: boolean;
  draggable?: boolean;
}

export default function ViewerProfilePicture({ circle, draggable }: Props) {
  const { handle } = useLens();

  const [firstLoad, setFirstLoad] = useState(true);

  const [{ data, fetching }] = useGetProfileQuery({
    variables: { request: { handles: [handle?.concat(HANDLE_ENDING)] } },
    pause: !handle,
  });

  useEffect(() => {
    if (firstLoad && !fetching) setFirstLoad(false);
  }, [firstLoad, fetching]);

  const profile = data?.profiles.items[0];
  const src = firstLoad
    ? null
    : getMediaURL(profile?.picture) ?? `https://avatar.tobi.sh/${handle}`;

  return <ProfilePicture circle={circle} draggable={draggable} src={src} />;
}
