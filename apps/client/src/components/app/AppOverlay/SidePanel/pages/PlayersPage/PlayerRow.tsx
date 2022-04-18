import { useProfile } from "ceramic";
import { useSetAtom } from "jotai";
import { Identity } from "../../../../helpers/types";
import { pageAtom, userAtom } from "../../helpers/atoms";

interface Props {
  id: string;
  viewerId: string;
  identity: Identity;
}

export default function PlayerRow({ id, viewerId, identity }: Props) {
  const setPage = useSetAtom(pageAtom);
  const setUser = useSetAtom(userAtom);

  const { profile } = useProfile(identity.did);

  const username = identity.isGuest
    ? `Guest-${id.substring(0, 6)}`
    : profile?.name ?? identity.did;
  const isViewer = viewerId === id;

  function handleNameClick() {
    if (identity.isGuest) return;
    setPage("User");
    setUser(identity.did);
  }

  return (
    <div key={id} className="flex space-x-1">
      <div
        onClick={handleNameClick}
        className={!identity.isGuest ? "hover:underline cursor-pointer" : ""}
      >
        {username}
      </div>
      {isViewer && <div className="text-neutral-500">(You)</div>}
    </div>
  );
}
