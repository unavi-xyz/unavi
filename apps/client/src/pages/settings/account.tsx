import { ChangeEvent, useEffect, useState } from "react";
import { useAccount } from "wagmi";

import { getSettingsLayout } from "../../home/layouts/SettingsLayout/SettingsLayout";
import { useProfilesByAddress } from "../../lib/lens/hooks/useProfilesByAddress";
import { useSetDefaultProfile } from "../../lib/lens/hooks/useSetDefaultProfile";
import { trimHandle } from "../../lib/lens/utils/trimHandle";
import Button from "../../ui/base/Button";
import Select from "../../ui/base/Select";
import MetaTags from "../../ui/MetaTags";

export default function Account() {
  const { address } = useAccount();

  const { profiles } = useProfilesByAddress(address);
  const defaultProfile = profiles?.find((profile) => profile.isDefault);

  const [options, setOptions] = useState<string[]>([]);
  const [selected, setSelected] = useState<string>();
  const [loading, setLoading] = useState(false);

  const setDefaultProfile = useSetDefaultProfile();

  const disabled =
    selected === `@${trimHandle(defaultProfile?.handle)}` && Boolean(selected);

  useEffect(() => {
    const handles =
      profiles?.map((profile) => `@${trimHandle(profile.handle)}`) ?? [];
    setOptions(handles);
    setSelected(handles[0]);
  }, [profiles]);

  useEffect(() => {
    if (defaultProfile) setSelected(`@${trimHandle(defaultProfile.handle)}`);
  }, [defaultProfile]);

  async function handleSave() {
    if (disabled || loading || !selected || !profiles) return;

    const handle = selected.slice(1);
    const profile = profiles.find(
      (profile) => trimHandle(profile.handle) === handle
    );

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
    <>
      <MetaTags title="Account" />

      <div
        className="bg-primaryContainer text-onPrimaryContainer space-y-8
                   rounded-3xl p-8 text-lg"
      >
        <div className="space-y-2">
          <div className="text-xl font-bold">Default Profile</div>
          <div>
            Selecting a default profile will help people discover who you are.
            You can change your default profile at any time.
          </div>
        </div>

        {defaultProfile && (
          <div className="flex space-x-1">
            <div>Current default profile:</div>
            <div className="font-black">
              @{trimHandle(defaultProfile.handle)}
            </div>
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

        <div className="flex w-full justify-end">
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
    </>
  );
}

Account.getLayout = getSettingsLayout;
