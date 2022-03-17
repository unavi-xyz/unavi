import { Dispatch, SetStateAction } from "react";
import { MdArrowBackIosNew } from "react-icons/md";

import { PACKS } from "../../../../helpers/editor/types";
import AssetCard from "./AssetCard";

interface Props {
  pack: string;
  setPack: Dispatch<SetStateAction<string>>;
}

export default function Assets({ pack, setPack }: Props) {
  const assets = PACKS[pack];

  return (
    <div className="space-y-6">
      <div className="flex items-center">
        <div className="w-1/3">
          <div
            onClick={() => setPack(undefined)}
            className="hover:cursor-pointer text-2xl p-3 rounded-full max-w-min"
          >
            <MdArrowBackIosNew />
          </div>
        </div>

        <div className="w-1/3 text-3xl flex justify-center">{pack}</div>
      </div>

      <div className="space-y-2">
        {assets.map((asset) => {
          return <AssetCard key={asset.name} asset={asset} />;
        })}
      </div>
    </div>
  );
}
