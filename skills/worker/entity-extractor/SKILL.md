---
id: ghost.worker.entity_extractor
name: entity-extractor
version: 0.1.0
kind: worker
domain: transformation
summary: Normalize evidence into structured entities, claims, dates, and relationships.
capabilities:
  - extract_entities
  - normalize
  - timeline_build
inputs:
  evidence:
    type: array
    required: true
  schema_hint:
    type: object
    required: false
outputs:
  entities:
    type: array
  claims:
    type: array
  timeline:
    type: array
tools:
  - read
  - grep
  - calc
constraints:
  - preserve source references for every extracted claim
  - do not change meaning while normalizing
handoff:
  can_delegate: false
  can_accept: true
telemetry:
  - entity_extracted
  - claim_normalized
  - timeline_point_added
---

# Entity Extractor

Purpose
- Convert raw evidence into machine-readable facts.
- Build a claim graph and timeline.

Execution spec
1. Accept source-backed evidence.
2. Extract people, orgs, products, dates, locations, and relationships.
3. Emit normalized ids.
4. Attach evidence references to every field.

Output contract
- entities, claims, relations, timeline, unresolved_fields.

Guardrails
- No invented facts.
- No final certainty labels.
- Keep source snippets alongside extracted structures.