import { useContext, useState } from "react";
import { AiFillHome, AiFillAppstore, AiOutlinePlus } from "react-icons/ai";
import { VscDebugDisconnect } from "react-icons/vsc";
import Link from "next/link";
import { CeramicContext, useProfile, useSpaces } from "ceramic";

import LoginDialog from "./LoginDialog/LoginDialog";
import CreateDialog from "./CreateDialog/CreateDialog";
import SpaceButton from "./SpaceButton";
import SidebarButton from "./SidebarButton";

export default function Sidebar() {
  const { authenticated, viewerId } = useContext(CeramicContext);

  const [openLogin, setOpenLogin] = useState(false);
  const [openCreate, setOpenCreate] = useState(false);

  const { imageUrl } = useProfile(viewerId);

  const spaces = useSpaces(viewerId);

  function handleCreate() {
    if (!authenticated) setOpenLogin(true);
    else setOpenCreate(true);
  }

  return (
    <div>
      <LoginDialog open={openLogin} setOpen={setOpenLogin} />
      <CreateDialog open={openCreate} setOpen={setOpenCreate} />

      <div
        className="fixed top-0 left-0 h-screen w-16 m-0 flex flex-col
                   justify-between bg-primary text-white shadow"
      >
        <div className="flex flex-col">
          <Link href="/" passHref>
            <span>
              <SidebarButton dark icon={<AiFillHome />} />
            </span>
          </Link>
          <Link href="/" passHref>
            <span>
              <SidebarButton dark icon={<AiFillAppstore />} />
            </span>
          </Link>

          <div>
            {spaces?.map((streamId) => {
              return <SpaceButton key={streamId} streamId={streamId} />;
            })}
          </div>

          <div onClick={handleCreate}>
            <SidebarButton selected={openCreate} icon={<AiOutlinePlus />} />
          </div>
        </div>

        <div className="flex flex-col space-y-4">
          <div className="bg-black bg-opacity-10">
            {authenticated ? (
              <Link href={`/user/${viewerId}`} passHref>
                <span>
                  <SidebarButton image={imageUrl} />
                </span>
              </Link>
            ) : (
              <div onClick={() => setOpenLogin(true)}>
                <SidebarButton dark icon={<VscDebugDisconnect />} />
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
