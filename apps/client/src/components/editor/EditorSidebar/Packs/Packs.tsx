import { Dispatch, SetStateAction } from "react";
import { HiOutlineCube } from "react-icons/hi";

import PackButton from "./PackButton";

interface Props {
  setPack: Dispatch<SetStateAction<string>>;
}

export default function Packs({ setPack }: Props) {
  return (
    <div className="space-y-6">
      <div className="text-2xl h-9 flex items-center justify-center">
        Object Packs
      </div>
      <div className="space-y-4">
        <PackButton
          name="Basic"
          icon={<HiOutlineCube />}
          onClick={() => setPack("Basic")}
        />
      </div>
    </div>
  );
}
