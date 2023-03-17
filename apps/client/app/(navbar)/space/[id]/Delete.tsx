"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";
import { Space__factory, SPACE_ADDRESS } from "contracts";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import { parseError } from "../../../../src/editor/utils/parseError";
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

    const toastId = "delete";

    setLoading(true);

    async function deleteSpace() {
      if (!signer) throw new Error("No signer");

      toast.loading("Waiting for signature...", { id: toastId });
      const contract = Space__factory.connect(SPACE_ADDRESS, signer);
      const tx = await contract.burn(id);

      toast.loading("Deleting from database...", { id: toastId });
      await deletePublication(id);

      toast.loading("Burning space...", { id: toastId });
      await tx.wait();
    }

    try {
      await deleteSpace();
      toast.success("Space deleted.", { id: toastId });

      router.push(`/user/${address}`);
    } catch (err) {
      console.error(err);
      toast.error(parseError(err, "Failed to delete space."), { id: toastId });
    }

    setLoading(false);
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
