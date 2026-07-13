import { Component, createResource, JSX, Show, splitProps } from "solid-js";

const CACHE_IMAGE_PATH = "/api/cache/image";
const CACHE_IMAGE_ID = /^[\dA-Fa-f]{64}$/;

const resolveImageUrl = async (url: string) => {
  if (CACHE_IMAGE_ID.test(url)) {
    return `${CACHE_IMAGE_PATH}?id=${url}`;
  }

  const parsed = new URL(url, globalThis.location.origin);

  if (parsed.origin === globalThis.location.origin && parsed.pathname === CACHE_IMAGE_PATH) {
    return url;
  }

  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    return url;
  }

  const response = await fetch(`${CACHE_IMAGE_PATH}?url=${encodeURIComponent(url)}`, { method: "POST" });

  if (!response.ok) {
    throw new Error(`Failed to cache image: ${response.status}`);
  }

  const location = response.headers.get("Location");

  if (!location) {
    throw new Error("Image cache response has no Location header");
  }

  return location;
};

type CachedImageProps = Omit<JSX.ImgHTMLAttributes<HTMLImageElement>, "src"> & {
  src: string;
};

export const CachedImage: Component<CachedImageProps> = (props) => {
  const [local, imageProps] = splitProps(props, ["src"]);
  const [source] = createResource(() => local.src, resolveImageUrl);

  return (
    <Show when={source()}>
      {src => <img {...imageProps} src={src()} />}
    </Show>
  );
};
