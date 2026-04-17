import { ActiveResearchWaveMonitor } from '../components/dashboard/active-research-wave-monitor';
import { LiveIntelligenceFeed } from '../components/dashboard/live-intelligence-feed';
import { ModelRoutingConfig } from '../components/dashboard/model-routing-config';
import { WorkerStatusGrid } from '../components/dashboard/worker-status-grid';
import { Badge, Button, Card } from '../components/ui';

export default function Page() {
  return (
    <main style={{ maxWidth: 1440, margin: '0 auto', padding: '28px 24px 40px', display: 'grid', gap: 24 }}>
      <header style={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'space-between', gap: 16, padding: '6px 0 8px' }}>
        <div style={{ display: 'grid', gap: 12 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, color: 'var(--ghost-text-soft)', fontSize: 12, letterSpacing: '0.18em', textTransform: 'uppercase' }}>
            <span style={{ width: 8, height: 8, borderRadius: 999, background: 'var(--ghost-accent)', boxShadow: '0 0 0 6px rgba(59,130,246,0.12)' }} />
            Ghost cockpit
          </div>
          <div style={{ display: 'grid', gap: 10, maxWidth: 860 }}>
            <h1 style={{ margin: 0, fontSize: 'clamp(38px, 5vw, 68px)', lineHeight: 0.98, letterSpacing: '-0.05em' }}>
              A monochrome command center for multi-model research.
            </h1>
            <p style={{ margin: 0, maxWidth: 760, color: 'var(--ghost-text-muted)', fontSize: 18, lineHeight: 1.7 }}>
              Built as a Next.js 16 cockpit with Bun-friendly scripts and shadcn-inspired surface components, tuned for a zinc palette, blue accents, and the fast dashboard cadence of Rasmic.xyz and Skills.sh.
            </p>
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 10 }}>
            <Badge tone="primary">Research wave 04</Badge>
            <Badge tone="default">Multi-model routing</Badge>
            <Badge tone="success">Evidence first</Badge>
          </div>
        </div>
        <Card style={{ width: 'min(420px, 100%)', padding: 20 }}>
          <div style={{ display: 'grid', gap: 12 }}>
            <div style={{ fontSize: 12, color: 'var(--ghost-text-soft)', textTransform: 'uppercase', letterSpacing: '0.12em' }}>Live snapshot</div>
            <div style={{ display: 'grid', gap: 8 }}>
              <div style={{ fontSize: 34, fontWeight: 800, letterSpacing: '-0.04em' }}>68% routed</div>
              <div style={{ color: 'var(--ghost-text-muted)', lineHeight: 1.6 }}>83 verified sources, 7 open contradictions, 1 synthesis pass in flight.</div>
            </div>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 10 }}>
              <Button tone="primary">Open live console</Button>
              <Button tone="secondary">Export snapshot</Button>
            </div>
          </div>
        </Card>
      </header>

      <section style={{ display: 'grid', gap: 24 }}>
        <div style={{ display: 'grid', gridTemplateColumns: 'minmax(0, 1.25fr) minmax(360px, 0.95fr)', gap: 24 }}>
          <ActiveResearchWaveMonitor />
          <ModelRoutingConfig />
        </div>
        <WorkerStatusGrid />
        <LiveIntelligenceFeed />
      </section>
    </main>
  );
}
