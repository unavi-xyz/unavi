import { useClient } from "@wired-labs/react-client";
import { MdArrowBack } from "react-icons/md";
import useSWR from "swr";

import { avatarFetcher } from "./avatarFetcher";
import { Nft, nftsResponseSchema } from "./schemas";
import { SettingsPage } from "./SettingsDialog";

const CHAIN_ID = 1;
const COLLECTION_ADDRESS = "0xc1def47cf1e15ee8c2a92f4e0e968372880d18d1";

interface Props {
  setPage: (page: SettingsPage) => void;
  onClose: () => void;
}

export default function AvatarBrowser({ setPage, onClose }: Props) {
  return (
    <div>
      <button
        onClick={() => setPage("Settings")}
        className="absolute top-[30px] left-8 rounded-full p-2 text-2xl transition hover:bg-neutral-200 active:bg-neutral-300"
      >
        <MdArrowBack />
      </button>

      <Avatars onClose={onClose} />
    </div>
  );
}

interface AvatarsProps {
  onClose: () => void;
}

function Avatars({ onClose }: AvatarsProps) {
  const { data, isLoading, error } = useSWR(
    `https://api.cryptoavatars.io/v1/nfts/avatars?license=CC0&chainId=${CHAIN_ID}&collectionAddress=${COLLECTION_ADDRESS}`,
    avatarFetcher
  );

  if (isLoading) return <div className="text-center">Loading...</div>;

  if (error) {
    console.error(error);
    return <div className="text-center text-red-900">Error fetching avatars</div>;
  }

  const parsed = nftsResponseSchema.safeParse(data);

  if (!parsed.success) {
    console.error(parsed.error);
    return <div className="text-center text-red-900">Error parsing data</div>;
  }

  const filteredNfts = parsed.data.nfts.filter((nft) => {
    if (!nft.metadata.asset) return false;

    if (nft.metadata.licenses?.redistribution) {
      return nft.metadata.licenses.redistribution === "Allow";
    }

    return true;
  });

  return (
    <div className="grid max-h-[700px] grid-cols-4 gap-3 overflow-y-auto p-2">
      {filteredNfts.map((nft) => {
        return <NftCard key={nft._id} nft={nft} onClose={onClose} />;
      })}
    </div>
  );
}

interface NftCardProps {
  nft: Nft;
  onClose: () => void;
}

function NftCard({ nft, onClose }: NftCardProps) {
  const { setAvatar } = useClient();

  const asset = nft.metadata.asset;
  const image = nft.metadata.image;

  if (!asset) return null;

  return (
    <button
      onClick={() => {
        setAvatar(asset);
        onClose();
      }}
      className="relative aspect-[7/10] rounded-xl bg-neutral-200 transition duration-100 ease-out hover:scale-105"
    >
      {image ? (
        <img
          src={image}
          alt=""
          crossOrigin="anonymous"
          className="absolute inset-0 h-full w-full rounded-xl object-cover"
        />
      ) : (
        <div className="text-lg">{nft.nftId}</div>
      )}
    </button>
  );
}
