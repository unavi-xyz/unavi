import { useGetProfileByHandleQuery } from "../../generated/graphql";
import { useLensStore } from "../../helpers/lens/store";
import ProfilePicture from "./ProfilePicture";

interface Props {
  circle?: boolean;
  draggable?: boolean;
}

export default function ViewerProfilePicture({ circle, draggable }: Props) {
  const handle = useLensStore((state) => state.handle);

  const [{ data }] = useGetProfileByHandleQuery({
    variables: { handle },
    pause: !handle,
  });

  return (
    <ProfilePicture
      circle={circle}
      draggable={draggable}
      profile={data?.profiles.items[0]}
    />
  );
}
