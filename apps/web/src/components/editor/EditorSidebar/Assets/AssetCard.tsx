import { Asset } from "3d";
import { useStore } from "../../../../helpers/editor/store";

interface Props {
  asset: Asset;
}

export default function AssetCard({ asset }: Props) {
  const newInstance = useStore((state) => state.newInstance);

  function handleClick() {
    newInstance(asset.name);
  }

  return (
    <div
      onClick={handleClick}
      className="bg-neutral-100 hover:bg-neutral-200 p-4 rounded hover:cursor-pointer"
    >
      {asset.name}
    </div>
  );
}
