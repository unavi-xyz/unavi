import { NextPageContext } from "next";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

import { useHidePublicationMutation } from "@wired-labs/lens";

import AvatarLayout from "../../../src/home/layouts/AvatarLayout/AvatarLayout";
import { getNavbarLayout } from "../../../src/home/layouts/NavbarLayout/NavbarLayout";
import { useLens } from "../../../src/lib/lens/hooks/useLens";
import {
  PublicationProps,
  getPublicationProps,
} from "../../../src/lib/lens/utils/getPublicationProps";
import Button from "../../../src/ui/base/Button";

export async function getServerSideProps({ res, query }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=120");

  const props = await getPublicationProps(query.id as string);

  return {
    props,
  };
}

export default function Settings(props: PublicationProps) {
  const router = useRouter();
  const id = router.query.id;

  const [loading, setLoading] = useState(false);
  const [, hidePublication] = useHidePublicationMutation();

  const { handle } = useLens();

  useEffect(() => {
    if (!handle && id) router.push(`/space/${id}`);
  }, [handle, id, router]);

  async function handleDelete() {
    if (loading) return;

    setLoading(true);

    try {
      await hidePublication({
        request: {
          publicationId: id as string,
        },
      });
      router.push(`/user/${handle}`);
    } catch (err) {
      console.error(err);
    }

    setLoading(false);
  }

  return (
    <AvatarLayout {...props}>
      <div className="bg-primaryContainer text-onPrimaryContainer rounded-2xl p-8 space-y-4">
        <div className="text-2xl font-bold">Danger Zone</div>

        <div className="text-lg">
          Deleting an avatar does not remove it from the blockchain. It only
          hides it from the indexer. Anyone can still find the avatar by using
          their own indexer.
        </div>

        <Button
          variant="filled"
          color="primary"
          squared="large"
          loading={loading}
          onClick={handleDelete}
        >
          Delete Avatar
        </Button>
      </div>
    </AvatarLayout>
  );
}

Settings.getLayout = getNavbarLayout;
