import { useHidePublicationMutation } from "lens";
import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

import { useLens } from "../../../client/lens/hooks/useLens";
import { getPublicationProps } from "../../../client/lens/utils/getPublicationProps";
import { trpc } from "../../../client/trpc";
import AvatarLayout from "../../../home/layouts/AvatarLayout/AvatarLayout";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import Button from "../../../ui/Button";

export const getServerSideProps = async ({ res, query }: GetServerSidePropsContext) => {
  const ONE_MINUTE_IN_SECONDS = 60;
  const ONE_WEEK_IN_SECONDS = 60 * 60 * 24 * 7;

  res.setHeader(
    "Cache-Control",
    `public, max-age=0, s-maxage=${ONE_MINUTE_IN_SECONDS}, stale-while-revalidate=${ONE_WEEK_IN_SECONDS}`
  );

  const id = query.id as string;
  const publicationProps = await getPublicationProps(id);

  return {
    props: {
      id,
      ...publicationProps,
    },
  };
};

export default function Settings(props: InferGetServerSidePropsType<typeof getServerSideProps>) {
  const router = useRouter();
  const id = router.query.id as string;

  const [loading, setLoading] = useState(false);
  const [, hidePublication] = useHidePublicationMutation();
  const { handle } = useLens();

  const { data: stats } = trpc.public.modelStats.useQuery(
    { publicationId: id },
    {
      refetchOnWindowFocus: false,
      refetchOnMount: false,
      refetchOnReconnect: false,
      trpc: {
        context: {
          skipBatch: true,
        },
      },
    }
  );

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
    <AvatarLayout {...props} stats={stats}>
      <div className="space-y-4 rounded-2xl bg-red-100 p-8 text-red-900">
        <div className="text-2xl font-bold">Danger Zone</div>

        <div className="text-lg">
          Deleting an avatar does not remove it from the blockchain. It only hides it from the
          indexer. Anyone can still find the avatar by using their own indexer.
        </div>

        <Button
          variant="filled"
          color="error"
          rounded="large"
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
