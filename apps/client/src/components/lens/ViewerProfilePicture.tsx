import { useGetProfileByHandleQuery } from "../../generated/graphql";
import { useLensStore } from "../../helpers/lens/store";

import ProfilePicture from "./ProfilePicture";

interface Props {
  circle?: boolean;
}

export default function ViewerProfilePicture({ circle }: Props) {
  const handle = useLensStore((state) => state.handle);

  const [{ data }] = useGetProfileByHandleQuery({
    variables: { handle },
    pause: !handle,
  });

  return <ProfilePicture circle={circle} profile={data?.profiles.items[0]} />;
}
