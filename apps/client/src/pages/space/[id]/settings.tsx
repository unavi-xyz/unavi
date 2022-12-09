import { useHidePublicationMutation } from "lens";
import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

import { useLens } from "../../../client/lens/hooks/useLens";
import { getPublicationProps } from "../../../client/lens/utils/getPublicationProps";
import { trpc } from "../../../client/trpc";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import SpaceLayout from "../../../home/layouts/SpaceLayout/SpaceLayout";
import Button from "../../../ui/Button";

export const getServerSideProps = async ({
  res,
  query,
}: GetServerSidePropsContext) => {
  const ONE_HOUR_IN_SECONDS = 60 * 60;
  const ONE_WEEK_IN_SECONDS = ONE_HOUR_IN_SECONDS * 24 * 7;

  res.setHeader(
    "Cache-Control",
    `public, s-maxage=${ONE_HOUR_IN_SECONDS}, stale-while-revalidate=${ONE_WEEK_IN_SECONDS}`
  );

  const props = await getPublicationProps(query.id as string);

  return {
    props,
  };
};

export default function Settings(
  props: InferGetServerSidePropsType<typeof getServerSideProps>
) {
  const [loading, setLoading] = useState(false);

  const { handle } = useLens();
  const [, hidePublication] = useHidePublicationMutation();
  const router = useRouter();
  const id = router.query.id as string;

  const { mutateAsync: deletePublication } =
    trpc.publication.delete.useMutation();

  useEffect(() => {
    if (!handle && id) router.push(`/space/${id}`);
  }, [handle, id, router]);

  async function handleDelete() {
    if (loading) return;

    setLoading(true);

    try {
      // Hide from lens API
      await hidePublication({ request: { publicationId: id } });

      // Remove from database
      await deletePublication({ lensId: id });

      router.push(`/user/${handle}`);
    } catch (err) {
      console.error(err);
    }

    setLoading(false);
  }

  return (
    <SpaceLayout {...props}>
      <div className="space-y-4 rounded-2xl bg-red-100 p-8 text-red-900">
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
