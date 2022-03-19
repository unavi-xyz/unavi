import { Dispatch, SetStateAction } from "react";
import { MdArrowBackIosNew } from "react-icons/md";

import { PACKS } from "../../helpers/types";
import AssetCard from "./AssetCard";
import GLTFCard from "./GLTFCard";

interface Props {
  pack: string;
  setPack: Dispatch<SetStateAction<string>>;
}

export default function Assets({ pack, setPack }: Props) {
  const assets = PACKS[pack];

  return (
    <div className="space-y-6">
      <div className="flex items-center h-9">
        <div className="w-1/3">
          <div
            onClick={() => setPack(undefined)}
            className="hover:cursor-pointer text-xl p-3 rounded-full max-w-min"
          >
            <MdArrowBackIosNew />
          </div>
        </div>

        <div className="w-1/3 text-2xl flex justify-center">{pack}</div>
      </div>

      <div className="grid gap-2 grid-cols-3">
        {assets.map((asset) => {
          return (
            <div key={asset.name} className="h-32">
              <AssetCard asset={asset} />
            </div>
          );
        })}

        {pack === "Basic" && <GLTFCard />}
      </div>
    </div>
  );
}
