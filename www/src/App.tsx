import { useState } from "react";
import { flamegraph_svg_from_json } from "engulf-wasm";

const sampleJson = `{
  "users": [
    { "type": "admin", "name": "Ada", "roles": ["root", "ops"] },
    { "type": "editor", "name": "Lee", "roles": ["content", "review"] }
  ],
  "metrics": { "requests": 1200, "errors": 12 }
}`;

export default function App() {
    const [svg, setSvg] = useState<string>("");
    const [error, setError] = useState<string>("");
    const [jsonText, setJsonText] = useState(sampleJson);

    const onSubmit = (event: React.FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setError("");
        setSvg("");

        let json: string;
        try {
            json = JSON.stringify(JSON.parse(jsonText));
        } catch (err) {
            setError(err instanceof Error ? err.message : String(err));
            return;
        }

        try {
            const result = flamegraph_svg_from_json(json, "engulf-wasm demo", ["type"]);
            setSvg(result);
        } catch (err) {
            setError(err instanceof Error ? err.message : String(err));
        }
    };

    return (
        <main style={{ padding: "2rem", fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace" }}>
            <h1>engulf-wasm demo</h1>
            <form onSubmit={onSubmit} style={{ marginBottom: "1rem" }}>
                <label htmlFor="json-input" style={{ display: "block", marginBottom: "0.5rem" }}>
                    JSON input
                </label>
                <textarea
                    id="json-input"
                    value={jsonText}
                    onChange={(event) => setJsonText(event.target.value)}
                    rows={12}
                    style={{ width: "100%", fontFamily: "inherit", fontSize: "0.9rem", padding: "0.75rem" }}
                />
                <div style={{ marginTop: "0.75rem", display: "flex", gap: "0.75rem" }}>
                    <button type="submit">Generate SVG</button>
                    <button type="button" onClick={() => setJsonText(sampleJson)}>
                        Reset sample
                    </button>
                </div>
            </form>
            {error ? <pre>{error}</pre> : null}
            {svg ? (
                <div aria-label="Flamegraph SVG" dangerouslySetInnerHTML={{ __html: svg }} />
            ) : (
                <p>Submit JSON to generate an SVG.</p>
            )}
        </main>
    );
}
