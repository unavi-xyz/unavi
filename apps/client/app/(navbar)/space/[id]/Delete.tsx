"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";
import { nanoid } from "nanoid";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { toast } from "react-hot-toast";
import { useContractWrite, useWalletClient } from "wagmi";
import { readContract } from "wagmi/actions";

import { getNFTSpace } from "@/app/api/nfts/[id]/space/helper";
import { deleteSpace } from "@/app/api/spaces/[id]/helper";
import { useAuth } from "@/src/client/AuthProvider";
import { SPACE_ADDRESS } from "@/src/contracts/addresses";
import { SPACE_ABI } from "@/src/contracts/SpaceAbi";
import { env } from "@/src/env.mjs";
import Button from "@/src/ui/Button";
import { parseError } from "@/src/utils/parseError";
import { SpaceId } from "@/src/utils/parseSpaceId";

interface Props {
  id: SpaceId;
}

export default function Delete({ id }: Props) {
  const { user } = useAuth();
  const router = useRouter();
  const [loading, setLoading] = useState(false);

  const { writeAsync: burn } = useContractWrite({
    abi: SPACE_ABI,
    address: SPACE_ADDRESS,
    functionName: "burn",
  });

  const { data: wallet } = useWalletClient();
  const { openConnectModal } = useConnectModal();

  async function handleDelete() {
    if (loading) return;

    const toastId = nanoid();

    setLoading(true);

    async function handleDelete() {
      if (id.type === "id") {
        toast.loading("Deleting space...", { id: toastId });
        await deleteSpace(id.value);
      } else {
        if (!wallet) {
          if (openConnectModal) openConnectModal();
          return;
        }

        const tokenURI = await readContract({
          abi: SPACE_ABI,
          address: SPACE_ADDRESS,
          args: [BigInt(id.value)],
          functionName: "tokenURI",
        });

        const nftsPath = `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/nfts/`;

        // Delete space NFT
        toast.loading("Waiting for signature...", { id: toastId });
        await burn({ args: [BigInt(id.value)] });

        toast.loading("Deleting space...", { id: toastId });

        if (tokenURI.startsWith(nftsPath)) {
          // Get database nft id from token URI
          const nftId = tokenURI
            .replace(nftsPath, "")
            .replace("/metadata.json", "");

          // Get space id
          const { spaceId } = await getNFTSpace(nftId);

          // Delete space from database
          await deleteSpace(spaceId);
        }
      }
    }

    try {
      await handleDelete();
      toast.success("Space deleted.", { id: toastId });

      router.push(`/@${user?.username}`);
    } catch (err) {
      console.error(err);
      toast.error(parseError(err, "Failed to delete space."), { id: toastId });
    }

    setLoading(false);
  }

  return (
    <div className="space-y-2 rounded-2xl bg-red-100 px-8 py-6 text-red-900 ring-2 ring-inset ring-red-900/20">
      <div className="text-2xl font-bold">Danger Zone</div>
      <div className="pb-1 text-lg">
        Deleting a space is permanent and cannot be undone.
      </div>

      <Button
        disabled={loading}
        onClick={handleDelete}
        className="rounded-xl bg-red-700"
      >
        Delete Space
      </Button>
    </div>
  );
}
