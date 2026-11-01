import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import App from "./App";

describe("App", () => {
  afterEach(() => invoke.mockReset());

  it("shows the name, the version from the core and that no universe is open", async () => {
    invoke.mockResolvedValue({ name: "Universe", version: "9.9.9" });
    render(<App />);
    expect(await screen.findByText("9.9.9")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Universe" })).toBeInTheDocument();
    expect(screen.getByText("No universe open")).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("app_info");
  });

  it("says the version is unavailable if the core can't be reached", async () => {
    invoke.mockRejectedValue(new Error("no core"));
    render(<App />);
    expect(await screen.findByText("Version unavailable")).toBeInTheDocument();
    expect(screen.getByText("No universe open")).toBeInTheDocument();
  });
});
