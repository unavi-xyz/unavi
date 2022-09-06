import { ChangeEvent, useState } from "react";
import { useAccount } from "wagmi";

import { useLens } from "../../../lib/lens/hooks/useLens";
import { useProfilesByAddress } from "../../../lib/lens/hooks/useProfilesByAddress";
import { trimHandle } from "../../../lib/lens/utils/trimHandle";
import Button from "../../../ui/base/Button";
import Select from "../../../ui/base/Select";

interface Props {
  onClose: () => void;
}

export default function SwitchProfilePage({ onClose }: Props) {
  const { switchProfile } = useLens();
  const { address } = useAccount();
  const { profiles } = useProfilesByAddress(address);
  const [selected, setSelected] = useState<string>();
  const handles =
    profiles?.map((profile) => `@${trimHandle(profile.handle)}`) ?? [];

  function handleSelect() {
    if (selected) {
      const rawHandle = selected.slice(1);
      switchProfile(rawHandle);
    }
    onClose();
  }

  return (
    <div className="space-y-4" onPointerUp={(e) => e.stopPropagation()}>
      <div className="text-3xl font-bold flex justify-center">
        Switch profile
      </div>

      <Select
        title="Select profile"
        options={handles}
        value={selected}
        onChange={(e: ChangeEvent<HTMLSelectElement>) =>
          setSelected(e.target.value)
        }
      />

      <div className="flex justify-end">
        <Button variant="tonal" onClick={handleSelect}>
          Select
        </Button>
      </div>
    </div>
  );
}
