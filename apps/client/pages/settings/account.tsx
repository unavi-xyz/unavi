import { ChangeEvent, useEffect, useState } from "react";

import { useEthersStore } from "../../src/helpers/ethers/store";
import { useProfilesByAddress } from "../../src/helpers/lens/hooks/useProfilesByAddress";
import { useDefaultProfileByAddress } from "../../src/helpers/lens/hooks/useDefaultProfileByAddress";
import { updateDefaultProfile } from "../../src/helpers/lens/actions/updateDefaultProfile";

import SettingsLayout from "../../src/components/layouts/SettingsLayout/SettingsLayout";
import Select from "../../src/components/base/Select";
import Button from "../../src/components/base/Button";

export default function Account() {
  const address = useEthersStore((state) => state.address);
  const profiles = useProfilesByAddress(address);
  const defaultProfile = useDefaultProfileByAddress(address);

  const [options, setOptions] = useState<string[]>([]);
  const [selected, setSelected] = useState<string>();

  const [loading, setLoading] = useState(false);

  const disabled =
    selected?.slice(1) === defaultProfile?.handle && Boolean(selected);

  useEffect(() => {
    const handles = profiles.map((profile) => `@${profile.handle}`);
    setOptions(handles);
  }, [profiles]);

  useEffect(() => {
    if (defaultProfile) setSelected(`@${defaultProfile.handle}`);
  }, [defaultProfile]);

  async function handleSave() {
    if (disabled || loading || !selected) return;

    const handle = selected.slice(1);

    setLoading(true);

    const profile = profiles.find((profile) => profile.handle === handle);
    if (!profile) return;

    //update the default profile
    try {
      await updateDefaultProfile(profile);
    } catch (err) {
      console.error(err);
      setLoading(false);
    }

    setLoading(false);
  }

  return (
    <div className="space-y-8 bg-white rounded-2xl border p-8">
      <div className="space-y-2">
        <div className="font-bold text-xl">Set default profile</div>
        <div className="text-neutral-500">
          Setting a default profile will help people discover who you are. You
          can change your default profile at any time.
        </div>
      </div>

      <div>
        <Select
          title="Select profile"
          options={options}
          value={selected}
          onChange={(e: ChangeEvent<HTMLSelectElement>) =>
            setSelected(e.target.value)
          }
        />
      </div>

      <Button onClick={handleSave} loading={loading} disabled={disabled}>
        Save
      </Button>
    </div>
  );
}

Account.Layout = SettingsLayout;
