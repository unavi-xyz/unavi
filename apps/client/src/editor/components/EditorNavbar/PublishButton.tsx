import { useState } from "react";

import Button from "../../../ui/Button";
import Dialog from "../../../ui/Dialog";
import PublishPage from "./PublishPage";

export default function PublishButton() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Dialog open={open} onClose={() => setOpen(false)}>
        <PublishPage />
      </Dialog>

      <Button variant="tonal" onClick={() => setOpen(true)}>
        Publish
      </Button>
    </>
  );
}
