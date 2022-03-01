import { Dispatch, SetStateAction, useState } from "react";

import Dialog from "../../../base/Dialog/Dialog";
import CreateDialogInfo from "./CreateSpaceDialogInfo";
import CreateDialogType from "./CreateSpaceDialogType";

interface Props {
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}
export default function CreateDialog({ open, setOpen }: Props) {
  const [type, setType] = useState("");

  return (
    <Dialog open={open} setOpen={setOpen}>
      {type === "public" ? (
        <CreateDialogInfo
          type={type}
          back={() => setType(undefined)}
          close={() => setOpen(false)}
        />
      ) : (
        <CreateDialogType setType={setType} />
      )}
    </Dialog>
  );
}
