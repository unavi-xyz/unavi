import { Asset } from "3d";
import { useAtom } from "jotai";

import { newInstance } from "../../../../helpers/editor/helpers";
import { sceneAtom } from "../../../../helpers/editor/state";

interface Props {
  asset: Asset;
}

export default function AssetCard({ asset }: Props) {
  const [scene, setScene] = useAtom(sceneAtom);

  function handleClick() {
    const newScene = newInstance(asset.name, scene);
    setScene(newScene);
  }

  return (
    <div
      onClick={handleClick}
      className="bg-neutral-100 hover:bg-neutral-200 p-4 rounded hover:cursor-pointer
                 ring-1 ring-neutral-400"
    >
      {asset.name}
    </div>
  );
}
