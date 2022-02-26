export async function uploadImageToIpfs(image: File) {
  const body = new FormData();
  body.append("path", image, image.name);
  const res = await fetch(`https://ipfs.infura.io:5001/api/v0/add`, {
    method: "POST",
    body,
  });
  const { Hash } = await res.json();
  return `ipfs://${Hash}`;
}

export function getImageUrl(hash: string) {
  const stripped = hash.replace("ipfs://", "");
  const imageUrl = `https://ipfs.infura.io:5001/api/v0/cat?arg=${stripped}`;
  return imageUrl;
}
