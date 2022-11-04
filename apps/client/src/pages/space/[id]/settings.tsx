import { useHidePublicationMutation } from "@wired-labs/lens";
import { GetServerSideProps, InferGetServerSidePropsType } from "next";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

import { useLens } from "../../../client/lens/hooks/useLens";
import { PublicationProps } from "../../../client/lens/utils/getPublicationProps";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import { getSpaceLayoutProps } from "../../../home/layouts/SpaceLayout/getSpaceLayoutProps";
import SpaceLayout from "../../../home/layouts/SpaceLayout/SpaceLayout";
import Button from "../../../ui/Button";

export const getServerSideProps: GetServerSideProps<
  PublicationProps & { host: string; playerCount: number | null }
> = async ({ res, query }) => {
  res.setHeader("Cache-Control", "s-maxage=30");

  const props = await getSpaceLayoutProps(query.id as string);

  return {
    props,
  };
};

export default function Settings(
  props: InferGetServerSidePropsType<typeof getServerSideProps>
) {
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
          publicationId: id,
        },
      });
      router.push(`/user/${handle}`);
    } catch (err) {
      console.error(err);
    }

    setLoading(false);
  }

  return (
    <SpaceLayout {...props}>
      <div className="space-y-4 rounded-2xl bg-errorContainer p-8 text-onErrorContainer">
        <div className="text-2xl font-bold">Danger Zone</div>

        <div className="text-lg">
          Deleting a space does not remove it from the blockchain. It only hides
          it from the indexer. Anyone can still find the space by using their
          own indexer.
        </div>

        <Button
          variant="filled"
          color="error"
          rounded="large"
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
