import { useContext, useState } from "react";
import { VscDebugDisconnect } from "react-icons/vsc";
import Link from "next/link";
import { CeramicContext, useProfile } from "ceramic";

import SidebarButton from "../SidebarButton/SidebarButton";
import SignInDialog from "./SignInDialog/SignInDialog";

export default function SignInButton() {
  const { authenticated, viewerId } = useContext(CeramicContext);
  const { imageUrl } = useProfile(viewerId);

  const [open, setOpen] = useState(false);

  return (
    <div>
      <SignInDialog open={open} setOpen={setOpen} />

      {authenticated ? (
        <Link href={`/user/${viewerId}`} passHref>
          <span>
            <SidebarButton image={imageUrl} />
          </span>
        </Link>
      ) : (
        <div onClick={() => setOpen(true)}>
          <SidebarButton dark icon={<VscDebugDisconnect />} />
        </div>
      )}
    </div>
  );
}
