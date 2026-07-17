import { defineConfig } from "astro/config";

const repository = process.env.GITHUB_REPOSITORY?.split("/")[1];

export default defineConfig({
  site: process.env.GITHUB_ACTIONS
    ? `https://${process.env.GITHUB_REPOSITORY_OWNER}.github.io`
    : "http://localhost:4321",
  base: process.env.GITHUB_ACTIONS && repository ? `/${repository}` : "/",
});
