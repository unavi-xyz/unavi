import { CubeTexture, ImageBitmapLoader } from "three";

// Can't use CubeTextureLoader in a worker, so load the texture manually
export async function loadCubeTexture(path: string): Promise<CubeTexture> {
  const loader = new ImageBitmapLoader();
  loader.setPath(path);

  const images = await Promise.all(
    ["right.bmp", "left.bmp", "top.bmp", "bottom.bmp", "front.bmp", "back.bmp"].map((file) =>
      loader.loadAsync(file)
    )
  );

  const texture = new CubeTexture();
  texture.needsUpdate = true;

  images.forEach((image, i) => {
    texture.images[i] = image;
  });

  return texture;
}
