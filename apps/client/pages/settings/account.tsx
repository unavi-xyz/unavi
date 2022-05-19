import { ChangeEvent, useEffect, useState } from "react";

import Button from "../../src/components/base/Button";
import Select from "../../src/components/base/Select";
import SettingsLayout from "../../src/components/layouts/SettingsLayout/SettingsLayout";
import { useEthersStore } from "../../src/helpers/ethers/store";
import { useProfilesByAddress } from "../../src/helpers/lens/hooks/useProfilesByAddress";
import { useSetDefaultProfile } from "../../src/helpers/lens/hooks/useSetDefaultProfile";

export default function Account() {
  const address = useEthersStore((state) => state.address);
  const profiles = useProfilesByAddress(address);
  const defaultProfile = profiles?.find((profile) => profile.isDefault);

  const [options, setOptions] = useState<string[]>([]);
  const [selected, setSelected] = useState<string>();
  const [loading, setLoading] = useState(false);

  const setDefaultProfile = useSetDefaultProfile();

  const disabled =
    selected?.slice(1) === defaultProfile?.handle && Boolean(selected);

  useEffect(() => {
    const handles = profiles?.map((profile) => `@${profile.handle}`) ?? [];
    setOptions(handles);
  }, [profiles]);

  useEffect(() => {
    if (defaultProfile) setSelected(`@${defaultProfile.handle}`);
  }, [defaultProfile]);

  async function handleSave() {
    if (disabled || loading || !selected || !profiles) return;

    const handle = selected.slice(1);
    const profile = profiles.find((profile) => profile.handle === handle);
    if (!profile) return;

    setLoading(true);

    try {
      await setDefaultProfile(profile.id);
    } catch (err) {
      console.error(err);
    }

    setLoading(false);
  }

  return (
    <div
      className="space-y-8 bg-primaryContainer text-onPrimaryContainer
                 rounded-3xl border p-8 text-lg"
    >
      <div className="space-y-2">
        <div className="font-bold text-xl">Default Profile</div>
        <div>
          Selecting a default profile will help people discover who you are. You
          can change your default profile at any time.
        </div>
      </div>

      {defaultProfile && (
        <div className="flex space-x-1">
          <div>Current default profile:</div>
          <div>@{defaultProfile.handle}</div>
        </div>
      )}

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

      <div className="w-full flex justify-end">
        <Button
          variant="filled"
          onClick={handleSave}
          loading={loading}
          disabled={disabled}
        >
          Save
        </Button>
      </div>
    </div>
  );
}

Account.Layout = SettingsLayout;
