"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";
import { Space__factory, SPACE_ADDRESS } from "contracts";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import Button from "../../../../src/ui/Button";
import { deletePublication } from "../../../api/publications/[id]/helper";

interface Props {
  id: number;
  address: string;
}

export default function Delete({ id, address }: Props) {
  const router = useRouter();
  const [loading, setLoading] = useState(false);

  const { data: signer } = useSigner();
  const { openConnectModal } = useConnectModal();

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

      // Remove from database
      await deletePublication(id);
    }

    toast
      .promise(deleteSpace(), {
        loading: "Deleting space...",
        success: "Space deleted",
        error: "Failed to delete space",
      })
      .then(() => router.push(`/user/${address}`))
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
