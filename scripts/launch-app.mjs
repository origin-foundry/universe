// Starts the optimised app built by `tauri build --no-bundle` (used by `npm run app`).
// Node rather than shell syntax, so it works whichever shell npm uses.
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import process from "node:process";

const exe = process.platform === "win32" ? "target/release/uni-app.exe" : "target/release/uni-app";
if (!existsSync(exe)) {
  process.stderr.write(`${exe} not found; the build step should have created it.\n`);
  process.exit(1);
}
spawn(exe, { stdio: "inherit" }).on("exit", (code) => process.exit(code ?? 0));
