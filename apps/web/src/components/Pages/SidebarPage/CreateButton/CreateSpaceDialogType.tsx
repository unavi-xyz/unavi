import { Dispatch, SetStateAction } from "react";
import { BiWorld } from "react-icons/bi";

import RichButton from "../../../base/RichButton";

interface Props {
  setType: Dispatch<SetStateAction<string>>;
}

export default function CreateDialogType({ setType }: Props) {
  return (
    <div className="flex flex-col space-y-4">
      <h1 className="text-3xl flex justify-center">Create a space</h1>
      <p className="text-lg flex justify-center opacity-80">
        Spaces are a way to group rooms and people. What kind of Space do you
        want to create? You can change this later.
      </p>

      <div
        onClick={() => setType("public")}
        className="flex flex-col space-y-4"
      >
        <RichButton
          icon={<BiWorld />}
          title="Public"
          description="Open space for anyone, best for communities"
        />
      </div>
    </div>
  );
}
