import { Badge, Button, Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Progress } from '../ui';

const checkpoints = [
  { label: 'Discovery', value: '4 probes queued', tone: 'success' },
  { label: 'Verification', value: '2 contradictions open', tone: 'warning' },
  { label: 'Synthesis', value: '1 brief pending', tone: 'primary' }
] as const;

const evidenceStats = [
  { label: 'sources traced', value: '83' },
  { label: 'claims verified', value: '51' },
  { label: 'escalations', value: '7' }
];

export function ActiveResearchWaveMonitor() {
  return (
    <Card>
      <CardHeader>
        <div style={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'space-between', gap: 12 }}>
          <div style={{ display: 'grid', gap: 10 }}>
            <Badge tone="primary">Active research wave</Badge>
            <CardTitle>Wave 04 · Market map + contradiction sweep</CardTitle>
            <CardDescription>
              The coordinator is pushing a high-confidence pass through discovery, verification, and synthesis while the cheapest sufficient models stay on the front line.
            </CardDescription>
          </div>
          <Badge tone="success">Live</Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div style={{ display: 'grid', gridTemplateColumns: 'minmax(0, 1.35fr) minmax(280px, 0.9fr)', gap: 20, alignItems: 'start' }}>
          <div style={{ display: 'grid', gap: 18 }}>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, minmax(0, 1fr))', gap: 12 }}>
              {evidenceStats.map((stat) => (
                <div key={stat.label} style={{ padding: 16, borderRadius: 18, background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.06)' }}>
                  <div style={{ fontSize: 12, color: 'var(--ghost-text-soft)', textTransform: 'uppercase', letterSpacing: '0.12em' }}>{stat.label}</div>
                  <div style={{ marginTop: 10, fontSize: 28, fontWeight: 700, letterSpacing: '-0.03em' }}>{stat.value}</div>
                </div>
              ))}
            </div>
            <div style={{ display: 'grid', gap: 10 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', gap: 12, fontSize: 13, color: 'var(--ghost-text-muted)' }}>
                <span>Wave progress</span>
                <span>68% complete</span>
              </div>
              <Progress value={68} />
            </div>
          </div>
          <div style={{ display: 'grid', gap: 12 }}>
            {checkpoints.map((checkpoint) => (
              <div key={checkpoint.label} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12, padding: 16, borderRadius: 18, background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.06)' }}>
                <div style={{ display: 'grid', gap: 4 }}>
                  <div style={{ fontWeight: 600 }}>{checkpoint.label}</div>
                  <div style={{ fontSize: 13, color: 'var(--ghost-text-muted)' }}>{checkpoint.value}</div>
                </div>
                <Badge tone={checkpoint.tone}>{checkpoint.label}</Badge>
              </div>
            ))}
          </div>
        </div>
      </CardContent>
      <CardFooter style={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'space-between', gap: 12 }}>
        <div style={{ color: 'var(--ghost-text-muted)', fontSize: 13 }}>Cadence: 2 minute checkpoints · reroute on provenance drift or source sparsity</div>
        <Button tone="secondary">Open wave log</Button>
      </CardFooter>
    </Card>
  );
}
