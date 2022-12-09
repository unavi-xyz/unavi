import { ChangeEvent, useEffect, useState } from "react";

import { useLogin } from "../../client/auth/LoginProvider";
import { useProfilesByAddress } from "../../client/lens/hooks/useProfilesByAddress";
import { useSetDefaultProfile } from "../../client/lens/hooks/useSetDefaultProfile";
import { trimHandle } from "../../client/lens/utils/trimHandle";
import { getSettingsLayout } from "../../home/layouts/SettingsLayout/SettingsLayout";
import MetaTags from "../../home/MetaTags";
import Button from "../../ui/Button";
import Select from "../../ui/Select";

export default function Account() {
  const { address } = useLogin();
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

      <div className="space-y-8 rounded-3xl bg-sky-100 p-8 text-lg">
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
