import { useState } from "react";
import { VscDebugDisconnect } from "react-icons/vsc";
import { useAuth, useProfile } from "ceramic";

import SignInDialog from "./SignInDialog";
import SidebarButton from "../SidebarButton";

export default function SignInButton() {
  const { authenticated, viewerId } = useAuth();
  const { profile, imageUrl } = useProfile(viewerId);

  const [open, setOpen] = useState(false);

  return (
    <div className="p-2">
      <SignInDialog open={open} setOpen={setOpen} />

      {authenticated ? (
        <SidebarButton tooltip={profile?.name} image={imageUrl} />
      ) : (
        <div onClick={() => setOpen(true)}>
          <SidebarButton tooltip="Sign In" dark icon={<VscDebugDisconnect />} />
        </div>
      )}
    </div>
  );
}
