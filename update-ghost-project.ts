import { saveProject } from '../mcp/linear-7bbcaaa4-2806-4795-8e76-621951163ffb.ts';
const description = `Ghost — OSINT-level Deep Research Harness

Ghost is a multi-stage research system: a coordinator plans the inquiry, narrow worker agents collect evidence, a verifier checks contradictions and provenance, and a reporter turns validated findings into a concise analyst brief. The MilksandMatcha insight reinforces this topology: multi-agent workflows remove the single-agent ceiling at the architecture level, so Ghost should not rely on one monolithic loop for discovery, verification, and synthesis.

Operational rules
- Route by task shape, not by habit
- Use the cheapest sufficient model for each subtask
- Require multi-source corroboration before promoting a claim
- Preserve blind spots and missed paths in the report footer
- Treat research as a repeatable, auditable pipeline rather than a one-shot answer`;
console.log(JSON.stringify(await saveProject({id:'6f5e27dc-bb83-47e5-b6f5-f3bcd0a46b13', description}), null, 2));
