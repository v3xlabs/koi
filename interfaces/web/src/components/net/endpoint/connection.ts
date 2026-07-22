export const endpointTypeForUrl = (endpointUrl: string) => {
    try {
        const protocol = new URL(endpointUrl).protocol;

        if (protocol === "http:" || protocol === "https:") return "http";

        if (protocol === "ws:" || protocol === "wss:") return "ws";
    }
    catch {
        return undefined;
    }

    return undefined;
};
