# Ghost architecture draft

Ghost is optimized for long-horizon, high-signal research rather than chatty assistance.

## Core design principles
- Separate discovery, verification, synthesis, and reporting
- Treat every claim as provisional until supported by multiple sources
- Prefer structured data extraction over raw browser automation
- Keep the reasoning loop stateful, but the outputs auditable
- Use the cheapest sufficient model for each step
- Preserve a full evidence trail with source, timestamp, confidence, and contradiction notes

## Multi-agent implication from the MilksandMatcha insight
The architecture should not assume one agent instance can do all work equally well.
Instead:
- a coordinator agent plans the task
- worker agents perform narrow retrieval and extraction
- a verifier agent checks contradictions and provenance
- a reporter agent converts validated findings into an analyst-ready brief

This avoids the single-agent ceiling and makes the system easier to reason about, test, and budget.

## Deep reasoning loop
1. Intake the objective and define what would count as a useful answer
2. Decompose the objective into source-specific workstreams
3. Route each workstream to the cheapest sufficient model or tool
4. Gather evidence in parallel where independence matters
5. Re-ask or re-route when a source conflicts, times out, or looks stale
6. Score confidence based on corroboration, freshness, and provenance quality
7. Emit a final report that includes what was found, what was not found, and what remains uncertain

## Reporting discipline
The reporting engine should not only summarize. It should also publish a constraint report:
- what the run maximized
- what the run sacrificed
- which sources were not examined
- which claims remain hypothesis-level
- what follow-up queries would close the largest gaps

This is the key guardrail that keeps Ghost from sounding confident while being under-verified.

## Reporting contract
Every report should include:
- answer summary
- supporting evidence
- contradictory evidence
- unresolved gaps
- source list
- confidence score
- suggested follow-up queries
