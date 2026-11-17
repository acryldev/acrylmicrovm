// Puts the acrylmicrovm CLI binary into each platform package under npm/ (built by `npm run artifacts`),
// so `npx acrylmicrovm` runs it through bin/acrylmicrovm.js. Usage: node scripts/add-cli-binary.mjs <path-to-binary>
import {
  chmodSync,
  copyFileSync,
  existsSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";

const binary = process.argv[2];
if (!binary || !existsSync(binary)) {
  console.error(`add-cli-binary: binary not found: ${binary}`);
  process.exit(1);
}
const dirs = readdirSync("npm");
if (dirs.length === 0) {
  console.error(
    "add-cli-binary: npm/ has no platform package; run `npm run artifacts` first",
  );
  process.exit(1);
}
for (const dir of dirs) {
  const target = join("npm", dir, "acrylmicrovm");
  copyFileSync(binary, target);
  chmodSync(target, 0o755);
  const manifest = join("npm", dir, "package.json");
  const pkg = JSON.parse(readFileSync(manifest, "utf8"));
  pkg.files = [...new Set([...(pkg.files ?? []), "acrylmicrovm"])];
  pkg.author = "WebBoxes s.r.o.";
  writeFileSync(manifest, `${JSON.stringify(pkg, null, 2)}\n`);
  console.log(`add-cli-binary: ${target}`);
}
