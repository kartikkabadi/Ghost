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

## Research loop
1. Intake user objective
2. Decompose into sub-questions
3. Route each sub-question to the best provider/tool
4. Gather evidence from multiple sources
5. Cross-check conflicts and stale references
6. Score confidence and identify blind spots
7. Emit a structured report with caveats and next actions

## Reporting contract
Every report should include:
- answer summary
- supporting evidence
- contradictory evidence
- unresolved gaps
- source list
- confidence score
- suggested follow-up queries
