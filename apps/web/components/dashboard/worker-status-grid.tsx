import { Badge, Card, CardContent, CardDescription, CardHeader, CardTitle, Progress } from '../ui';

const workers = [
  {
    name: 'Planner',
    model: 'Qwen 2.5',
    state: 'routing requests',
    badge: 'coordination',
    tone: 'primary',
    queue: '6 jobs',
    latency: '220 ms',
    throughput: '14/min',
    progress: 74
  },
  {
    name: 'Collector',
    model: 'MiniMax',
    state: 'pulling evidence',
    badge: 'collection',
    tone: 'success',
    queue: '18 jobs',
    latency: '480 ms',
    throughput: '31/min',
    progress: 82
  },
  {
    name: 'Verifier',
    model: 'Gemini 3 Flash',
    state: 'checking claims',
    badge: 'verification',
    tone: 'warning',
    queue: '4 jobs',
    latency: '360 ms',
    throughput: '9/min',
    progress: 61
  },
  {
    name: 'Reporter',
    model: 'Gemini 3 Flash',
    state: 'assembling brief',
    badge: 'synthesis',
    tone: 'default',
    queue: '2 jobs',
    latency: '190 ms',
    throughput: '5/min',
    progress: 47
  }
] as const;

export function WorkerStatusGrid() {
  return (
    <Card>
      <CardHeader>
        <Badge tone="default">Worker status grid</Badge>
        <CardTitle>Agent lane health across the orchestration stack</CardTitle>
        <CardDescription>
          Each worker exposes queue depth, response speed, and phase ownership so the cockpit can quickly show where the bottleneck moved.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, minmax(0, 1fr))', gap: 16 }}>
          {workers.map((worker) => (
            <article key={worker.name} style={{ padding: 18, borderRadius: 20, background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.06)', display: 'grid', gap: 16 }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12 }}>
                <div style={{ display: 'grid', gap: 6 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                    <div style={{ fontSize: 18, fontWeight: 700 }}>{worker.name}</div>
                    <Badge tone={worker.tone}>{worker.badge}</Badge>
                  </div>
                  <div style={{ fontSize: 13, color: 'var(--ghost-text-muted)' }}>{worker.model} · {worker.state}</div>
                </div>
                <div style={{ width: 12, height: 12, borderRadius: 999, background: 'var(--ghost-accent)', boxShadow: '0 0 0 6px rgba(59,130,246,0.14)' }} />
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, minmax(0, 1fr))', gap: 10 }}>
                {[
                  ['Queue', worker.queue],
                  ['Latency', worker.latency],
                  ['Throughput', worker.throughput]
                ].map(([label, value]) => (
                  <div key={label} style={{ padding: 12, borderRadius: 16, background: 'rgba(0,0,0,0.18)', border: '1px solid rgba(255,255,255,0.04)' }}>
                    <div style={{ fontSize: 11, color: 'var(--ghost-text-soft)', textTransform: 'uppercase', letterSpacing: '0.12em' }}>{label}</div>
                    <div style={{ marginTop: 8, fontSize: 15, fontWeight: 600 }}>{value}</div>
                  </div>
                ))}
              </div>
              <div style={{ display: 'grid', gap: 8 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', gap: 10, fontSize: 13, color: 'var(--ghost-text-muted)' }}>
                  <span>Lane capacity</span>
                  <span>{worker.progress}% utilized</span>
                </div>
                <Progress value={worker.progress} />
              </div>
            </article>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
