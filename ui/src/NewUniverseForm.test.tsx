import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import NewUniverseForm from "./NewUniverseForm";

const NAME_MESSAGE = "A universe needs a name of 1 to 200 characters.";

function setup() {
  const onCreate = vi.fn(async () => undefined);
  const onCancel = vi.fn();
  render(<NewUniverseForm onCreate={onCreate} onCancel={onCancel} />);
  const input = screen.getByLabelText("Name");
  const create = screen.getByRole("button", { name: "Create" });
  return { onCreate, onCancel, input, create };
}

describe("NewUniverseForm", () => {
  it("rejects an empty or blank name", () => {
    const { onCreate, input, create } = setup();
    fireEvent.click(create);
    expect(screen.getByRole("alert")).toHaveTextContent(NAME_MESSAGE);
    fireEvent.change(input, { target: { value: "   " } });
    fireEvent.click(create);
    expect(screen.getByRole("alert")).toHaveTextContent(NAME_MESSAGE);
    expect(onCreate).not.toHaveBeenCalled();
  });

  it("rejects a name longer than 200 characters, counting characters not UTF-16 units", () => {
    const { onCreate, input, create } = setup();
    fireEvent.change(input, { target: { value: "✶".repeat(201) } });
    fireEvent.click(create);
    expect(screen.getByRole("alert")).toHaveTextContent(NAME_MESSAGE);
    expect(onCreate).not.toHaveBeenCalled();

    fireEvent.change(input, { target: { value: "𝔄".repeat(200) } }); // two UTF-16 units each
    fireEvent.click(create);
    expect(onCreate).toHaveBeenCalledWith("𝔄".repeat(200));
  });

  it("creates the universe with the trimmed name", () => {
    const { onCreate, input, create } = setup();
    fireEvent.change(input, { target: { value: "  Aster ✶ Ñandú  " } });
    fireEvent.click(create);
    expect(onCreate).toHaveBeenCalledWith("Aster ✶ Ñandú");
  });

  it("can be cancelled", () => {
    const { onCancel } = setup();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancel).toHaveBeenCalled();
  });
});
