import { useEffect, useState } from "react";
import { AiOutlinePlus } from "react-icons/ai";
import { IoMdCheckmark } from "react-icons/io";
import { BsFillGearFill } from "react-icons/bs";
import { joinSpace, leaveSpace, useAuth, useSpace, useSpaces } from "ceramic";

import Tooltip from "../../base/Tooltip";
import NavbarButton from "./NavbarButton";

interface Props {
  spaceId: string;
}

export default function SaveButton({ spaceId }: Props) {
  const { authenticated, viewerId } = useAuth();
  const { controller } = useSpace(spaceId);
  const spaces = useSpaces(viewerId);

  const [saved, setSaved] = useState(false);
  const [owner, setOwner] = useState(false);

  useEffect(() => {
    if (controller === viewerId) setOwner(true);
    else setOwner(false);
  }, [controller, viewerId]);

  useEffect(() => {
    const found = spaces?.find((streamId) => streamId === spaceId);

    if (found) {
      setSaved(true);
    }
  }, [spaceId, spaces]);

  async function handleSave() {
    await joinSpace(spaceId);
    setSaved(true);
  }

  async function handleRemove() {
    await leaveSpace(spaceId);
    setSaved(false);
  }

  if (!authenticated) return null;

  return (
    <div className="group">
      {owner ? (
        <NavbarButton>
          <BsFillGearFill />
        </NavbarButton>
      ) : saved ? (
        <Tooltip text="Saved">
          <NavbarButton onClick={handleRemove}>
            <IoMdCheckmark />
          </NavbarButton>
        </Tooltip>
      ) : (
        <Tooltip text="Save">
          <NavbarButton onClick={handleSave}>
            <AiOutlinePlus />
          </NavbarButton>
        </Tooltip>
      )}
    </div>
  );
}
