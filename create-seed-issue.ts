import { saveIssue } from '../mcp/linear-7bbcaaa4-2806-4795-8e76-621951163ffb.ts';
const description = `Design the Ghost research topology so it explicitly avoids the single-agent ceiling.

Context:
- A coordinator should decompose the objective.
- Worker agents should gather and normalize evidence.
- A verifier should cross-check claims, detect contradictions, and mark stale sources.
- A reporter should emit a concise, auditable brief for the analyst.

Acceptance criteria:
- The architecture treats multi-agent composition as a first-class primitive.
- The deep reasoning loop preserves provenance and blind-spot reporting.
- The final report separates confirmed findings from hypotheses and missing coverage.
- The implementation path remains compatible with a Rust core, TypeScript UI, and SQLite persistence.`;
console.log(JSON.stringify(await saveIssue({team:'Kartik', project:'Ghost', title:'Ghost architecture: coordinator-worker-verifier-reporter topology', description}), null, 2));
