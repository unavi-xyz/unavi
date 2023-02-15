import { useConnectModal } from "@rainbow-me/rainbowkit";
import { Space__factory, SPACE_ADDRESS } from "contracts";
import { useRouter } from "next/router";
import { useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import { useSession } from "../client/auth/useSession";
import { trpc } from "../client/trpc";
import Button from "../ui/Button";

interface Props {
  id: number;
}

export default function SpaceSettings({ id }: Props) {
  const { mutate: deletePublication } = trpc.publication.delete.useMutation();

  const [loading, setLoading] = useState(false);

  const { data: session } = useSession();
  const router = useRouter();
  const { data: signer } = useSigner();
  const { openConnectModal } = useConnectModal();
  const utils = trpc.useContext();

  async function handleDelete() {
    if (loading) return;

    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    setLoading(true);

    async function deleteSpace() {
      if (!signer) throw new Error("No signer");

      // Burn NFT
      const contract = Space__factory.connect(SPACE_ADDRESS, signer);
      await contract.burn(id);

      await Promise.all([
        // Remove from database
        deletePublication({ spaceId: id }),
        // Invalidate trpc cache
        utils.space.byId.invalidate({ id }),
        utils.space.latest.invalidate({ owner: session?.address ?? "" }),
        utils.project.getAll.invalidate(),
      ]);
    }

    toast
      .promise(deleteSpace(), {
        loading: "Deleting space...",
        success: "Space deleted",
        error: "Failed to delete space",
      })
      .then(() => router.push(`/user/${session?.address}`))
      .catch((err) => console.error(err))
      .finally(() => setLoading(false));
  }

  return (
    <div className="space-y-2 rounded-2xl bg-red-100 px-8 py-6 text-red-900 ring-2 ring-inset ring-red-900/20">
      <div className="text-2xl font-bold">Danger Zone</div>
      <div className="pb-1 text-lg">Deleting a space is permanent and cannot be undone.</div>

      <Button disabled={loading} onClick={handleDelete} className="rounded-lg bg-red-700">
        Delete Space
      </Button>
    </div>
  );
}
