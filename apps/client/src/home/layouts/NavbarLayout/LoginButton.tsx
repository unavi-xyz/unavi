import { useState } from "react";

import Button from "../../../ui/base/Button";
import Dialog from "../../../ui/base/Dialog";
import LoginPage from "./LoginPage";

export default function LoginButton() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Dialog open={open} onClose={() => setOpen(false)}>
        <LoginPage />
      </Dialog>

      <Button variant="filled" onClick={() => setOpen(true)}>
        Login
      </Button>
    </>
  );
}
