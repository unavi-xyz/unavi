import { useEffect, useState } from "react";
import { VscDebugDisconnect } from "react-icons/vsc";

import SignInPage from "../../../components/SidebarLayout/Sidebar/BottomButton/SignInPage";
import { Dialog, Tooltip } from "../../../components/base";
import { useStore } from "../../helpers/store";

export default function SignInButton() {
  const isPointerLocked = useStore((state) => state.isPointerLocked);

  const [openSignIn, setOpenSignIn] = useState(false);

  useEffect(() => {
    if (isPointerLocked && openSignIn) setOpenSignIn(false);
  }, [isPointerLocked, openSignIn]);

  return (
    <>
      <Dialog open={openSignIn} setOpen={setOpenSignIn}>
        <div onClick={(e) => e.stopPropagation()}>
          <SignInPage />
        </div>
      </Dialog>

      <Tooltip text="Sign in">
        <div
          onClick={(e) => {
            e.stopPropagation();
            setOpenSignIn(true);
          }}
          className="bg-white rounded-lg flex items-center justify-center
                     w-12 h-12 cursor-pointer hover:bg-neutral-100 z-10"
        >
          <VscDebugDisconnect className="text-xl" />
        </div>
      </Tooltip>
    </>
  );
}
