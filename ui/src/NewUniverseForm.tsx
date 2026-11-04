import { useState, type FormEvent } from "react";

/** Same rule and message as `UniverseName` in crates/uni-store/src/name.rs. */
export const MAX_NAME_CHARS = 200;
export const NAME_MESSAGE = "A universe needs a name of 1 to 200 characters.";

/** Counts Unicode scalar values, as Rust does, rather than UTF-16 units. */
function charCount(text: string): number {
  return [...text].length;
}

interface Props {
  /** Creates the universe; rejects with a message to show if the core refuses. */
  onCreate: (name: string) => Promise<unknown>;
  onCancel: () => void;
}

export default function NewUniverseForm({ onCreate, onCancel }: Props) {
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);

  async function submit(event: FormEvent) {
    event.preventDefault();
    const trimmed = name.trim();
    const length = charCount(trimmed);
    if (length < 1 || length > MAX_NAME_CHARS) {
      setError(NAME_MESSAGE);
      return;
    }
    setError(null);
    try {
      await onCreate(trimmed);
    } catch (refused) {
      setError(String(refused));
    }
  }

  return (
    <form className="new-universe" onSubmit={submit}>
      <h2>New universe</h2>
      <label htmlFor="universe-name">Name</label>
      <input
        id="universe-name"
        autoFocus
        value={name}
        onChange={(event) => setName(event.target.value)}
      />
      {error !== null && <p role="alert">{error}</p>}
      <div className="actions">
        <button type="submit">Create</button>
        <button type="button" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </form>
  );
}
