import { Badge, Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Slider, Switch } from '../ui';

const lanes = [
  { name: 'Qwen 2.5', role: 'planning + cheap inference', score: 84, tone: 'primary' },
  { name: 'MiniMax', role: 'fast web retrieval', score: 73, tone: 'success' },
  { name: 'Gemini 3 Flash', role: 'verification + synthesis', score: 91, tone: 'warning' }
] as const;

export function ModelRoutingConfig() {
  return (
    <Card>
      <CardHeader>
        <Badge tone="primary">Model routing config</Badge>
        <CardTitle>Cost-aware routing policy for the research loop</CardTitle>
        <CardDescription>
          The cockpit keeps routing rules visible so model choice, latency limits, and escalation thresholds stay auditable while the wave is active.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div style={{ display: 'grid', gridTemplateColumns: 'minmax(0, 1fr) minmax(300px, 0.92fr)', gap: 18 }}>
          <div style={{ display: 'grid', gap: 12 }}>
            {lanes.map((lane) => (
              <div key={lane.name} style={{ padding: 16, borderRadius: 18, background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.06)', display: 'grid', gap: 10 }}>
                <div style={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'space-between', gap: 12 }}>
                  <div style={{ display: 'grid', gap: 4 }}>
                    <div style={{ fontSize: 16, fontWeight: 700 }}>{lane.name}</div>
                    <div style={{ fontSize: 13, color: 'var(--ghost-text-muted)' }}>{lane.role}</div>
                  </div>
                  <Badge tone={lane.tone}>{lane.score}% fit</Badge>
                </div>
                <Slider value={lane.score} label="Route confidence" />
              </div>
            ))}
          </div>
          <div style={{ display: 'grid', gap: 12 }}>
            <Switch checked label="Prefer cheapest sufficient model" />
            <Switch checked label="Escalate on contradiction drift" />
            <Switch checked={false} label="Allow direct synthesis bypass" />
            <div style={{ padding: 16, borderRadius: 18, background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.06)', display: 'grid', gap: 14 }}>
              <div style={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'space-between', gap: 12 }}>
                <div style={{ display: 'grid', gap: 4 }}>
                  <div style={{ fontSize: 16, fontWeight: 700 }}>Routing guardrails</div>
                  <div style={{ fontSize: 13, color: 'var(--ghost-text-muted)' }}>Latency cap: 600 ms · budget cap: medium · provenance required: yes</div>
                </div>
                <Badge tone="default">Stable</Badge>
              </div>
              <div style={{ display: 'grid', gap: 10 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', gap: 12, fontSize: 13, color: 'var(--ghost-text-muted)' }}>
                  <span>Escalation threshold</span>
                  <span>72%</span>
                </div>
                <Slider value={72} />
              </div>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 10 }}>
                <Button tone="primary">Apply route snapshot</Button>
                <Button tone="secondary">Review fallback path</Button>
              </div>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
