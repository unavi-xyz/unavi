import { Dispatch, SetStateAction } from "react";
import { HiOutlineCube } from "react-icons/hi";
import { FaGamepad } from "react-icons/fa";

import PackButton from "./PackButton";

interface Props {
  setPack: Dispatch<SetStateAction<string>>;
}

export default function AssetPacks({ setPack }: Props) {
  return (
    <div className="space-y-6">
      <div className="text-3xl">Asset Packs</div>
      <div className="space-y-4">
        <PackButton
          name="Basic"
          icon={<HiOutlineCube />}
          onClick={() => setPack("Basic")}
        />
        <PackButton
          name="Game"
          icon={<FaGamepad />}
          onClick={() => setPack("Game")}
        />
      </div>
    </div>
  );
}
