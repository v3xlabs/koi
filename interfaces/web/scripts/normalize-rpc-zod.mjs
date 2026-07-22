import { readFile, writeFile } from "node:fs/promises";
import { URL } from "node:url";

const outputPath = new URL("../src/api/bindings.zod.gen.ts", import.meta.url);
const generated = await readFile(outputPath, "utf8");
const normalized = generated
    .replaceAll(".optional().nullable()", ".nullish().transform(value => value ?? undefined)")
    .replaceAll(".optional()", ".nullish().transform(value => value ?? undefined)");

await writeFile(outputPath, normalized);
