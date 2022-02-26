import { useState } from "react";
import { AiFillHome, AiFillAppstore, AiOutlinePlus } from "react-icons/ai";
import Link from "next/link";
import { useAuth, useSpaces } from "ceramic";

import CreateDialog from "./CreateDialog/CreateDialog";
import SpaceButton from "./SpaceButton/SpaceButton";
import SidebarButton from "./SidebarButton/SidebarButton";
import SignInButton from "./SignInButton/SignInButton";

export default function Sidebar() {
  const { authenticated, viewerId } = useAuth();

  const [openCreate, setOpenCreate] = useState(false);

  const spaces = useSpaces(viewerId);

  return (
    <div>
      <CreateDialog open={openCreate} setOpen={setOpenCreate} />

      <div
        className="fixed top-0 left-0 h-screen w-16 m-0 flex flex-col
                   justify-between bg-primary text-white shadow"
      >
        <div className="flex flex-col">
          <Link href="/" passHref>
            <span>
              <SidebarButton tooltip="Home" dark icon={<AiFillHome />} />
            </span>
          </Link>
          <Link href="/" passHref>
            <span>
              <SidebarButton tooltip="Rooms" dark icon={<AiFillAppstore />} />
            </span>
          </Link>

          <div>
            {spaces?.map((streamId) => {
              return <SpaceButton key={streamId} streamId={streamId} />;
            })}
          </div>

          {authenticated && (
            <div onClick={() => setOpenCreate(true)}>
              <SidebarButton
                tooltip="Create"
                selected={openCreate}
                icon={<AiOutlinePlus />}
              />
            </div>
          )}
        </div>

        <div className="flex flex-col space-y-4">
          <div className="bg-black bg-opacity-10">
            <SignInButton />
          </div>
        </div>
      </div>
    </div>
  );
}
