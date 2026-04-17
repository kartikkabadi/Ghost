import { Badge, Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui';

const feedItems = [
  {
    time: '09:18',
    source: 'Research probe · web',
    confidence: '96%',
    tone: 'success',
    headline: 'New signal cluster found in the latest crawl',
    detail: 'Three articles converge on the same pricing pattern; one outlier was downgraded after source checks.'
  },
  {
    time: '09:12',
    source: 'Verifier · contradiction check',
    confidence: '84%',
    tone: 'warning',
    headline: 'Two claims need secondary corroboration',
    detail: 'Both claims rely on a single primary source and a cached summary. Added a follow-up evidence sweep.'
  },
  {
    time: '09:05',
    source: 'Reporter · synthesis',
    confidence: '92%',
    tone: 'primary',
    headline: 'Draft brief updated with provenance notes',
    detail: 'The report now preserves blind spots, source depth, and unresolved follow-up questions for the analyst footer.'
  },
  {
    time: '08:58',
    source: 'Model router',
    confidence: '100%',
    tone: 'default',
    headline: 'Routing stayed inside the cheap-first policy',
    detail: 'No escalation to the heavier lane was needed; the wave remained within budget and latency guardrails.'
  }
] as const;

export function LiveIntelligenceFeed() {
  return (
    <Card>
      <CardHeader>
        <div style={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'space-between', gap: 12 }}>
          <div style={{ display: 'grid', gap: 8 }}>
            <Badge tone="primary">Live intelligence feed</Badge>
            <CardTitle>Streaming evidence, verification notes, and routing updates</CardTitle>
            <CardDescription>
              A compact, analyst-friendly feed inspired by the content density of Rasmic and Skills while keeping the cockpit focused on the current wave.
            </CardDescription>
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
            <Badge tone="default">All signals</Badge>
            <Badge tone="success">High confidence</Badge>
            <Badge tone="warning">Follow-up needed</Badge>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div style={{ display: 'grid', gap: 12 }}>
          {feedItems.map((item) => (
            <article key={item.headline} style={{ display: 'grid', gridTemplateColumns: '84px minmax(0, 1fr)', gap: 14, padding: 16, borderRadius: 18, background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.06)' }}>
              <div style={{ display: 'grid', gap: 8, alignContent: 'start' }}>
                <div style={{ fontSize: 18, fontWeight: 700, letterSpacing: '-0.02em' }}>{item.time}</div>
                <Badge tone={item.tone}>{item.confidence}</Badge>
              </div>
              <div style={{ display: 'grid', gap: 8 }}>
                <div style={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: 10 }}>
                  <div style={{ fontSize: 13, color: 'var(--ghost-text-soft)', textTransform: 'uppercase', letterSpacing: '0.12em' }}>{item.source}</div>
                </div>
                <div style={{ fontSize: 16, fontWeight: 700, letterSpacing: '-0.02em' }}>{item.headline}</div>
                <div style={{ fontSize: 14, lineHeight: 1.65, color: 'var(--ghost-text-muted)' }}>{item.detail}</div>
              </div>
            </article>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
