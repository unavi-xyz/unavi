import { useGetProfileByHandleQuery } from "../../generated/graphql";
import { HANDLE_ENDING } from "../../helpers/lens/constants";
import { useMediaImage } from "../../helpers/lens/hooks/useMediaImage";
import { useLensStore } from "../../helpers/lens/store";
import ProfilePicture from "./ProfilePicture";

interface Props {
  circle?: boolean;
  draggable?: boolean;
}

export default function ViewerProfilePicture({ circle, draggable }: Props) {
  const handle = useLensStore((state) => state.handle);

  const [{ data }] = useGetProfileByHandleQuery({
    variables: { handle: handle?.concat(HANDLE_ENDING) },
    pause: !handle,
  });

  const profile = data?.profiles.items[0];
  const src = useMediaImage(profile?.picture);

  return <ProfilePicture circle={circle} draggable={draggable} src={src} />;
}
