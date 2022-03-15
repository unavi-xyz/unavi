import { useState } from "react";
import { VscDebugDisconnect } from "react-icons/vsc";

import SidebarButton, { Colors } from "../SidebarButton";
import SignInDialog from "./SignInDialog";

export default function SignInButton() {
  const [open, setOpen] = useState(false);

  return (
    <div>
      <SignInDialog open={open} setOpen={setOpen} />

      <div onClick={() => setOpen(true)}>
        <SidebarButton
          icon={<VscDebugDisconnect />}
          text="Sign In"
          color={Colors.red}
        />
      </div>
    </div>
  );
}
