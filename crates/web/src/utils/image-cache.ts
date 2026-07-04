const CACHE_IMAGE_PATH = "/api/cache/image";

export const cacheImageUrl = (url: string | undefined | null) => {
  if (!url) {
    return undefined;
  }

  const parsed = new URL(url, globalThis.location.origin);

  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    return url;
  }

  return `${CACHE_IMAGE_PATH}?url=${
    encodeURIComponent(url)
  }`;
};
