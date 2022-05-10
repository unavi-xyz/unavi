import { useState } from "react";

import Button from "../../base/Button";
import Dialog from "../../base/Dialog";
import PublishPage from "./PublishPage";

export default function PublishButton() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Dialog open={open} onClose={() => setOpen(false)}>
        <PublishPage />
      </Dialog>

      <Button variant="filled" onClick={() => setOpen(true)}>
        Publish
      </Button>
    </>
  );
}
