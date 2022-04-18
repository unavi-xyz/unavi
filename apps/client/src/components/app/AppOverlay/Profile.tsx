import { useState } from "react";
import { VscDebugDisconnect } from "react-icons/vsc";
import { useAuth, useIpfsFile, useProfile } from "ceramic";

import SignInPage from "../../../layouts/SidebarLayout/Sidebar/SignInButton/SignInPage";
import { Dialog } from "../../base";

export default function Profile() {
  const { viewerId, authenticated } = useAuth();
  const { profile } = useProfile(viewerId);
  const { url } = useIpfsFile(profile?.image?.original.src);

  const [openSignIn, setOpenSignIn] = useState(false);

  if (!authenticated) {
    return (
      <>
        <Dialog open={openSignIn} setOpen={setOpenSignIn}>
          <div onClick={(e) => e.stopPropagation()}>
            <SignInPage />
          </div>
        </Dialog>

        <div className="flex flex-col m-4">
          <div
            onClick={(e) => {
              e.stopPropagation();
              setOpenSignIn(true);
            }}
            className="rounded-full z-10 cursor-pointer p-0.5 bg-position-move
                       bg-gradient-to-r from-primary to-secondary"
          >
            <div
              className="bg-white rounded-full flex items-center justify-center
                         px-6 py-3 space-x-3"
            >
              <VscDebugDisconnect className="text-lg" />
              <div>Sign in</div>
            </div>
          </div>
        </div>
      </>
    );
  }

  return (
    <div className="flex flex-col w-full max-w-sm m-4">
      <div
        onClick={(e) => e.stopPropagation()}
        className="bg-white rounded-2xl flex items-center space-x-4 p-2 z-10"
      >
        <div>
          {url && (
            <img
              src={url}
              alt="profile picture"
              className="object-cover h-14 w-14 rounded-full"
            />
          )}
        </div>

        <div className="flex flex-col justify-center">
          <div className="text-lg">{profile?.name}</div>
          <div className="text-sm w-64 overflow-hidden text-ellipsis text-neutral-500">
            {viewerId}
          </div>
        </div>
      </div>
    </div>
  );
}
