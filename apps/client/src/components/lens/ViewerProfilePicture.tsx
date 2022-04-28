import { useProfileByHandle } from "../../helpers/lens/hooks/useProfileByHandle";
import { useLensStore } from "../../helpers/lens/store";

import ProfilePicture from "./ProfilePicture";

interface Props {
  circle?: boolean;
}

export default function ViewerProfilePicture({ circle }: Props) {
  const handle = useLensStore((state) => state.handle);
  const profile = useProfileByHandle(handle);

  if (!profile) return null;

  return <ProfilePicture circle={circle} profile={profile} />;
}
