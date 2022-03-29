import { useState } from "react";
import { VscDebugDisconnect } from "react-icons/vsc";
import { Dialog } from "../../../../components/base";

import SidebarButton, { Colors } from "../SidebarButton";
import SignInPage from "./SignInPage";

export default function SignInButton() {
  const [open, setOpen] = useState(false);

  return (
    <div>
      <Dialog open={open} setOpen={setOpen}>
        <SignInPage />
      </Dialog>

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
