"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";
import { WorldMetadata } from "@wired-protocol/types";
import { ERC721Metadata, Space__factory, SPACE_ADDRESS } from "contracts";
import { nanoid } from "nanoid";
import { useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import { updateSpace } from "@/app/api/spaces/[id]/helper";
import { mintSpace } from "@/app/api/spaces/[id]/mint/helper";
import { getSpaceNFTFileUpload } from "@/app/api/spaces/[id]/nft/files/[file]/helper";
import { useAuth } from "@/src/client/AuthProvider";
import { env } from "@/src/env.mjs";
import { parseError } from "@/src/studio/utils/parseError";
import Button from "@/src/ui/Button";
import { SpaceDBId } from "@/src/utils/parseSpaceId";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";
import { toHex } from "@/src/utils/toHex";

interface Props {
  id: SpaceDBId;
  metadata: WorldMetadata;
}

export default function Mint({ id, metadata }: Props) {
  const [loading, setLoading] = useState(false);

  const { data: signer } = useSigner();
  const { openConnectModal } = useConnectModal();
  const { user } = useAuth();

  async function handleMint() {
    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    if (!user) return;

    setLoading(true);

    const toastId = nanoid();

    async function uploadMetadata(tokenId: number | undefined) {
      const erc721metadata: ERC721Metadata = {
        animation_url: metadata.model,
        description: metadata.info?.description,
        external_url: tokenId
          ? `${env.NEXT_PUBLIC_DEPLOYED_URL}/space/${toHex(tokenId)}`
          : undefined,
        image: metadata.info?.image,
        name: metadata.info?.name || `Space ${id.value.slice(0, 6)}`,
      };

      // Upload to S3
      const url = await getSpaceNFTFileUpload(id.value, "metadata");

      const response = await fetch(url, {
        method: "PUT",
        body: JSON.stringify(erc721metadata),
        headers: {
          "Content-Type": "application/json",
          "x-amz-acl": "public-read",
        },
      });

      if (!response.ok) throw new Error("Failed to upload metadata");
    }

    try {
      toast.loading("Preparing transaction...", { id: toastId });
      const { nftId } = await mintSpace(id.value);

      // Mint space NFT
      toast.loading("Waiting for signature...", { id: toastId });
      const metadataURI = cdnURL(S3Path.spaceNFT(nftId).metadata);
      const contract = Space__factory.connect(SPACE_ADDRESS, signer);
      const tx = await contract.mintWithTokenURI(metadataURI);

      toast.loading("Minting space...", { id: toastId });
      await tx.wait();

      // Wait for token to be indexed
      await new Promise((resolve) => setTimeout(resolve, 5000));

      let attempts = 0;

      const findTokenId = async (): Promise<number> => {
        // Loop backwards through past 10 tokens to find the one we just minted
        const count = (await contract.count()).toNumber();
        const max = Math.min(count, 10);
        let i = 0;

        while (i < max) {
          i++;
          const nextId = count - i;

          const owner = await contract.ownerOf(nextId);
          if (owner !== user.address) continue;

          const uri = await contract.tokenURI(nextId);
          if (uri !== metadataURI) continue;

          return nextId;
        }

        attempts++;
        if (attempts > 4) throw new Error("Failed to find token ID");

        return new Promise((resolve) => setTimeout(() => resolve(findTokenId()), 1000));
      };

      const tokenId = await findTokenId();

      toast.loading("Uploading metadata...", { id: toastId });

      await Promise.all([
        // Upload metadata
        uploadMetadata(tokenId),
        // Link NFT to space
        updateSpace(id.value, { tokenId }),
      ]);

      toast.success("Space minted!", { id: toastId });
    } catch (e) {
      toast.error(parseError(e, "Failed to mint NFT"), { id: toastId });
      console.error(e);
    }

    setLoading(false);
  }

  return (
    <div className="space-y-2 rounded-2xl bg-sky-100 px-8 py-6 text-sky-900 ring-2 ring-inset ring-sky-900/20">
      <div className="text-2xl font-bold">Mint NFT</div>
      <div className="pb-1 text-lg">
        Mint the space as an NFT, making it easier for others to discover.
      </div>

      <Button disabled={loading || !user} onClick={handleMint} className="rounded-xl bg-sky-700">
        Mint
      </Button>
    </div>
  );
}
