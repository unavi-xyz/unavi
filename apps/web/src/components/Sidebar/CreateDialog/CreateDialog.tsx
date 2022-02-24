import { Dispatch, SetStateAction, useState } from "react";

import Dialog from "../../Dialog";
import CreateDialogInfo from "./CreateDialogInfo";
import CreateDialogType from "./CreateDialogType";

interface Props {
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}
export default function CreateDialog({ open, setOpen }: Props) {
  const [type, setType] = useState("");

  return (
    <Dialog open={open} setOpen={setOpen}>
      {type === "public" ? (
        <CreateDialogInfo type={type} back={() => setType(undefined)} />
      ) : (
        <CreateDialogType setType={setType} />
      )}
    </Dialog>
  );
}
