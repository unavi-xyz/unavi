import { useContext, useState } from "react";
import { AiFillHome, AiFillAppstore, AiOutlinePlus } from "react-icons/ai";
import { VscDebugDisconnect } from "react-icons/vsc";
import { BiArrowToRight } from "react-icons/bi";
import Link from "next/link";
import { CeramicContext, useProfile, useSpaces } from "ceramic";

import LoginDialog from "./LoginDialog/LoginDialog";
import CreateDialog from "./CreateDialog/CreateDialog";
import SpaceButton from "./SpaceButton";

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
          <div className="px-2">
            <div
              className="flex justify-center py-2 text-xl hover:bg-black
                         hover:bg-opacity-20 hover:cursor-pointer rounded-xl
                         transition-all duration-150"
            >
              <BiArrowToRight />
            </div>
          </div>

          <div className="bg-black bg-opacity-10">
            {authenticated ? (
              <Link href={`/user/${viewerId}`} passHref>
                <img
                  className="object-cover relative flex items-center justify-center h-12 w-12 my-2 mx-auto
                             shadow-lg bg-neutral-800 hover:bg-neutral-700 hover:cursor-pointer
                             rounded-3xl text-xl"
                  src={imageUrl}
                  alt=""
                />
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

function SidebarButton({ icon = <></>, selected = false, dark = false }) {
  const colors = dark
    ? "bg-neutral-800 hover:bg-neutral-700 text-white"
    : "bg-white hover:bg-neutral-300 text-neutral-800";

  const round = selected ? "rounded-xl" : "rounded-3xl";

  return (
    <div
      className={`relative flex items-center justify-center h-12 w-12 my-2 mx-auto
                  shadow-lg hover:cursor-pointer hover:rounded-xl transition-all
                  ease-linear duration-150 text-xl ${colors} ${round}`}
    >
      {icon}
    </div>
  );
}
