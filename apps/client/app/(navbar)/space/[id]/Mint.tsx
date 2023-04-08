"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";
import { ERC721Metadata, Space__factory, SPACE_ADDRESS } from "contracts";
import { useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import { mintSpace } from "@/app/api/spaces/[id]/mint/helper";
import { getSpaceNFTFileUpload } from "@/app/api/spaces/[id]/nft/files/[file]/helper";
import { useSession } from "@/src/client/auth/useSession";
import { parseError } from "@/src/editor/utils/parseError";
import { env } from "@/src/env.mjs";
import { useProfileByAddress } from "@/src/play/hooks/useProfileByAddress";
import { SpaceMetadata } from "@/src/server/helpers/readSpaceMetadata";
import Button from "@/src/ui/Button";
import { SpaceDBId } from "@/src/utils/parseSpaceId";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";
import { toHex } from "@/src/utils/toHex";

interface Props {
  id: SpaceDBId;
  metadata: SpaceMetadata;
}

export default function Mint({ id, metadata }: Props) {
  const [loading, setLoading] = useState(false);

  const { data: signer } = useSigner();
  const { openConnectModal } = useConnectModal();
  const { data: session } = useSession();
  const { profile } = useProfileByAddress(session?.address);

  async function handleMint() {
    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    if (!session?.address) throw new Error("Session not found");

    setLoading(true);

    const toastId = "mint";

    async function uploadMetadata(tokenId: number | undefined) {
      if (!session?.address) throw new Error("Session not found");

      const erc721metadata: ERC721Metadata = {
        animation_url: metadata.uri,
        description: metadata.description,
        external_url: tokenId
          ? `${env.NEXT_PUBLIC_DEPLOYED_URL}/space/${toHex(tokenId)}`
          : `${env.NEXT_PUBLIC_DEPLOYED_URL}/user/${profile ? toHex(profile.id) : session.address}`,
        image: metadata.image,
        name: metadata.title,
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

      // Get token ID
      // Loop backwards through all spaces until we find the matching content URI
      const count = (await contract.count()).toNumber();
      let i = 0;
      let tokenId: number | undefined;

      while (tokenId === undefined && i < count) {
        i++;
        const nextId = count - i;

        const owner = await contract.ownerOf(nextId);
        if (owner !== session.address) continue;

        const uri = await contract.tokenURI(nextId);
        if (uri !== metadataURI) continue;

        tokenId = nextId;
      }

      // Upload metadata
      toast.loading("Uploading metadata...", { id: toastId });
      await uploadMetadata(tokenId);

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

      <Button disabled={loading} onClick={handleMint} className="rounded-xl bg-sky-700">
        Mint
      </Button>
    </div>
  );
}
