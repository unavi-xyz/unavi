import { useState } from "react";

import Button from "../../base/Button";
import Dialog from "../../base/Dialog";
import LoginPage from "./LoginPage";

export default function LoginButton() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Dialog open={open} onClose={() => setOpen(false)}>
        <LoginPage />
      </Dialog>

      <Button onClick={() => setOpen(true)}>Login</Button>
    </>
  );
}
