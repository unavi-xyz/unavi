import { NetworkingProvider } from "../networking";

export interface SpaceProps {
  spaceId: string;
  host: string;
}

export default function Space({ spaceId, host }: SpaceProps) {
  return <NetworkingProvider spaceId={spaceId} host={host}></NetworkingProvider>;
}
