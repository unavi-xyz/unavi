import { NextPageContext } from "next";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

import { useHidePublicationMutation } from "../../../src/generated/graphql";
import { getNavbarLayout } from "../../../src/home/layouts/NavbarLayout/NavbarLayout";
import SpaceLayout from "../../../src/home/layouts/SpaceLayout/SpaceLayout";
import {
  SpaceLayoutProps,
  getSpaceLayoutProps,
} from "../../../src/home/layouts/SpaceLayout/getSpaceLayoutProps";
import { authenticate } from "../../../src/lib/lens/authentication";
import { useLensStore } from "../../../src/lib/lens/store";
import Button from "../../../src/ui/base/Button";

export async function getServerSideProps({ res, query }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=10");

  const props = await getSpaceLayoutProps(query.id as string);

  return {
    props,
  };
}

export default function Settings(props: SpaceLayoutProps) {
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
    <SpaceLayout {...props}>
      <div className="bg-primaryContainer text-onPrimaryContainer rounded-2xl p-8 space-y-4">
        <div className="text-2xl font-bold">Danger Zone</div>

        <div className="text-lg">
          Deleting a space does not remove it from the blockchain. It only hides
          it from the indexer. Anyone can still find the space by using their
          own indexer.
        </div>

        <Button
          variant="filled"
          color="primary"
          squared="large"
          loading={loading}
          onClick={handleDelete}
        >
          Delete Space
        </Button>
      </div>
    </SpaceLayout>
  );
}

Settings.getLayout = getNavbarLayout;
