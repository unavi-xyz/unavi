import { Dispatch, SetStateAction, useRef } from "react";
import { Button, Dialog, TextField } from "../base";

interface Props {
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export function NewWorldDialog({ open, setOpen }: Props) {
  const name = useRef<HTMLInputElement>();

  function handleCreate() {}

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="space-y-6">
        <h1 className="text-3xl flex justify-center">Create a World</h1>

        <div className="space-y-4">
          <TextField title="Name" inputRef={name} defaultValue="New world" />
          <TextField title="Description" inputRef={name} />
        </div>

        <Button onClick={handleCreate}>
          <div>Create</div>
        </Button>
      </div>
    </Dialog>
  );
}
