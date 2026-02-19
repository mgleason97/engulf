import { useEffect, useState } from "react";
import init, { flamegraph_svg_from_json } from "engulf-wasm";

const sampleJson = {
  users: [
    { type: "admin", name: "Ada", roles: ["root", "ops"] },
    { type: "editor", name: "Lee", roles: ["content", "review"] },
  ],
  metrics: { requests: 1200, errors: 12 },
};

export default function App() {
  const [svg, setSvg] = useState<string>("");
  const [error, setError] = useState<string>("");

  useEffect(() => {
    let active = true;

    const run = async () => {
      try {
        await init();
        const json = JSON.stringify(sampleJson);
        const result = flamegraph_svg_from_json(json, "engulf-wasm demo", ["type"]);
        if (active) {
          setSvg(result);
        }
      } catch (err) {
        if (active) {
          setError(err instanceof Error ? err.message : String(err));
        }
      }
    };

    void run();

    return () => {
      active = false;
    };
  }, []);

  if (error) {
    return (
      <main style={{ padding: "2rem", fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace" }}>
        <h1>engulf-wasm demo</h1>
        <pre>{error}</pre>
      </main>
    );
  }

  return (
    <main style={{ padding: "2rem", fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace" }}>
      <h1>engulf-wasm demo</h1>
      {svg ? (
        <div
          aria-label="Flamegraph SVG"
          dangerouslySetInnerHTML={{ __html: svg }}
        />
      ) : (
        <p>Generating SVG...</p>
      )}
    </main>
  );
}
