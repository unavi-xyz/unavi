import { useContext } from "react";
import { ImSpinner2 } from "react-icons/im";
import { IpfsContext } from "ceramic";

export default function IPFSLoadingIndicator() {
  const { ipfs } = useContext(IpfsContext);

  if (ipfs) return null;

  return (
    <div className="flex items-center space-x-2 text-sm text-neutral-500">
      <ImSpinner2 className="animate-spin" />

      <div>Connecting to IPFS...</div>
    </div>
  );
}
