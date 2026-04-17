# Ghost skills directory

This directory contains capability packages for Ghost pre-configured agents.

Conventions
- Each skill lives in its own folder.
- Each skill uses a SKILL.md file with YAML frontmatter plus implementation guidance.
- Frontmatter is the source of truth for the router.
- The markdown body is the human-operable spec.
- Skills should expose typed inputs and typed outputs.
- Skills should declare whether they are coordinator, worker, verifier, or reporter roles.
- Skills should state tool permissions, evidence policy, and handoff behavior explicitly.

Suggested layout
- skills/coordinator/<skill-name>/SKILL.md
- skills/worker/<skill-name>/SKILL.md
- skills/verifier/<skill-name>/SKILL.md
- skills/reporter/<skill-name>/SKILL.md
- skills/_schema/skill.schema.json

Runtime expectations
- Parse frontmatter before activation.
- Validate tool policy and output schema before execution.
- Return structured artifacts, evidence, blockers, and confidence.
- Preserve provenance and contradiction notes across handoffs.

Example skill families included here
- research coordinator
- web search worker
- entity extraction worker
- claim verification worker
- analyst reporter