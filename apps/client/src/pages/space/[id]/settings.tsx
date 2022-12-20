import { useConnectModal } from "@rainbow-me/rainbowkit";
import { Space__factory, SPACE_ADDRESS } from "contracts";
import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";
import { useRouter } from "next/router";
import { useState } from "react";
import { useSigner } from "wagmi";

import { useSession } from "../../../client/auth/useSession";
import { trpc } from "../../../client/trpc";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import SpaceLayout from "../../../home/layouts/SpaceLayout/SpaceLayout";
import Button from "../../../ui/Button";
import { hexDisplayToNumber } from "../../../utils/numberToHexDisplay";

export const getServerSideProps = async ({ res, query }: GetServerSidePropsContext) => {
  const ONE_MINUTE_IN_SECONDS = 60;
  const ONE_WEEK_IN_SECONDS = 60 * 60 * 24 * 7;

  res.setHeader(
    "Cache-Control",
    `public, max-age=0, s-maxage=${ONE_MINUTE_IN_SECONDS}, stale-while-revalidate=${ONE_WEEK_IN_SECONDS}`
  );

  const hexId = query.id as string;
  const id = hexDisplayToNumber(hexId);

  return {
    props: { id },
  };
};

export default function Settings({ id }: InferGetServerSidePropsType<typeof getServerSideProps>) {
  const [loading, setLoading] = useState(false);

  const { data: session } = useSession();
  const router = useRouter();
  const { data: signer } = useSigner();
  const { openConnectModal } = useConnectModal();

  const { data: space } = trpc.space.byId.useQuery({ id });

  async function handleDelete() {
    if (loading) return;

    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    setLoading(true);

    try {
      // Burn NFT
      const contract = Space__factory.connect(SPACE_ADDRESS, signer);

      // await contract.burn(id);

      // Remove from database
      // await deletePublication({ lensId: id });

      router.push(`/user/${session?.address}`);
    } catch (err) {
      console.error(err);
    }

    setLoading(false);
  }

  return (
    <SpaceLayout id={id} author={space?.author ?? null} metadata={space?.metadata ?? null}>
      <div className="space-y-2 rounded-2xl bg-red-100 p-8 text-red-900">
        <div className="text-2xl font-bold">Danger Zone</div>

        <div className="pb-1 text-lg">Deleting a space is permanent and cannot be undone.</div>

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
