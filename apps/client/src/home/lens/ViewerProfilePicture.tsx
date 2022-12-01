import { useGetProfileQuery } from "lens";
import { useEffect, useState } from "react";

import { HANDLE_ENDING } from "../../client/lens/constants";
import { useLens } from "../../client/lens/hooks/useLens";
import { getMediaURL } from "../../utils/getMediaURL";
import ProfilePicture from "./ProfilePicture";

interface Props {
  circle?: boolean;
  draggable?: boolean;
  size: number;
}

export default function ViewerProfilePicture({
  circle,
  draggable,
  size,
}: Props) {
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
  const src = firstLoad ? null : getMediaURL(profile?.picture);

  if (!handle) return null;

  return (
    <ProfilePicture
      circle={circle}
      draggable={draggable}
      src={src}
      uniqueKey={handle}
      size={size}
    />
  );
}
