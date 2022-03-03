import { Dispatch, SetStateAction } from "react";
import { FaCrown } from "react-icons/fa";
import Link from "next/link";
import { useProfile, useSpace } from "ceramic";

import Dialog from "../../../base/Dialog/Dialog";
import Tooltip from "../../../base/Tooltip";

interface Props {
  spaceId: string;
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}
export default function InfoDialog({ spaceId, open, setOpen }: Props) {
  const { space, controller } = useSpace(spaceId);
  const { profile } = useProfile(controller);

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="flex flex-col space-y-4">
        <h1 className="text-3xl flex justify-center font-medium">
          {space?.name}
        </h1>

        <div className="flex items-center justify-center space-x-2 text-lg">
          <Tooltip text="Owner">
            <FaCrown className="text-xl hover:cursor-help" />
          </Tooltip>

          <div className="hover:cursor-pointer hover:underline">
            <Link href={`/user/${controller}`}>
              {profile?.name ?? controller}
            </Link>
          </div>
        </div>

        <hr className="border-neutral-200" />

        <div className="text-lg">{space?.description}</div>
      </div>
    </Dialog>
  );
}
