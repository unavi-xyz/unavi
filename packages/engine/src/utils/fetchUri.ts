import { parseUri, UriType } from "./parseUri";

export async function fetchUri(uri: string, baseUrl: string | null = null) {
  // Parse uri
  const type = parseUri(uri);

  let url: string;
  switch (type) {
    case UriType.DataUri:
      const data = uri.split(",")[1];
      const buffer = Buffer.from(data, "base64");
      const blob = new Blob([buffer]);
      url = URL.createObjectURL(blob);
      break;
    case UriType.IPFS:
      throw new Error("IPFS is not yet supported");
    case UriType.Url:
      url = uri;
      break;
    case UriType.RelativePath:
      // Append base url if present
      url = baseUrl ? `${baseUrl}/${uri}` : uri;
      break;
    default:
      throw new Error("Unknown URI type");
  }

  // Fetch
  const res = await fetch(url);
  return res;
}
