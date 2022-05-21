import { useRouter } from "next/router";
import { useEffect, useState } from "react";

import Button from "../../../src/components/base/Button";
import { getSpaceLayout } from "../../../src/components/layouts/SpaceLayout/SpaceLayout";
import { useHidePublicationMutation } from "../../../src/generated/graphql";
import { authenticate } from "../../../src/helpers/lens/authentication";
import { useLensStore } from "../../../src/helpers/lens/store";

export default function Settings() {
  const router = useRouter();
  const id = router.query.id;

  const handle = useLensStore((state) => state.handle);
  const [loading, setLoading] = useState(false);
  const [, hidePublication] = useHidePublicationMutation();

  useEffect(() => {
    if (!handle && id) router.push(`/space/${id}`);
  }, [handle, id, router]);

  async function handleDelete() {
    if (loading) return;

    setLoading(true);

    try {
      await authenticate();
      await hidePublication({
        publicationId: id,
      });
      router.push(`/user/${handle}`);
    } catch (err) {
      console.error(err);
    }

    setLoading(false);
  }

  return (
    <div className="bg-tertiaryContainer text-onTertiaryContainer rounded-2xl p-8 space-y-4">
      <div className="text-2xl font-bold">Danger Zone</div>

      <div className="text-lg">
        Deleting a space does not remove it from the blockchain. It only hides
        it from the indexer. Anyone can still find the space by using their own
        indexer.
      </div>

      <Button
        variant="filled"
        color="tertiary"
        loading={loading}
        squared
        onClick={handleDelete}
      >
        Delete Space
      </Button>
    </div>
  );
}

Settings.getLayout = getSpaceLayout;
