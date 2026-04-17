export default function Page() {
  return (
    <main style={{ padding: 32, maxWidth: 1200, margin: "0 auto" }}>
      <div style={{ display: "grid", gap: 16 }}>
        <div style={{ fontSize: 12, letterSpacing: 2, textTransform: "uppercase", color: "#9ca3af" }}>Ghost OSINT Harness</div>
        <h1 style={{ fontSize: 52, lineHeight: 1.05, margin: 0 }}>Autonomous research with evidence-first reporting.</h1>
        <p style={{ maxWidth: 760, color: "#d1d5db", fontSize: 18, lineHeight: 1.7 }}>
          Provider gateway, orchestration engine, SQLite evidence store, and analyst review UI.
          Built to route simple work cheaply, push complex work into structured multi-agent passes,
          and report what was found, what was verified, and what remains uncertain.
        </p>
      </div>
    </main>
  );
}
