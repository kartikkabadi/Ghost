export type ProviderKind = "llm" | "search" | "crawl" | "extraction" | "enrichment";

export interface SourceRef {
  id: string;
  url: string;
  title?: string;
  fetchedAt: string;
  provider: string;
  kind: ProviderKind;
}

export interface EvidenceClaim {
  id: string;
  claim: string;
  confidence: number;
  sources: SourceRef[];
  contradictions: string[];
}

export interface ResearchReport {
  objective: string;
  summary: string;
  claims: EvidenceClaim[];
  gaps: string[];
  nextQueries: string[];
  generatedAt: string;
}

export type ByokProviderKind = "openrouter" | "xai" | "firecrawl" | "exa";
export type ProviderCapability = "plan" | "reason" | "search" | "crawl" | "extract" | "enrich" | "report";
export type TaskKind = "plan" | "search" | "crawl" | "extract" | "enrich" | "verify" | "synthesize" | "report" | { custom: string };

export interface GatewayRequest {
  provider: ByokProviderKind;
  operation: string;
  method: string;
  url: string;
  headers: Record<string, string>;
  body: unknown;
}

export interface GatewayResponse {
  provider: ByokProviderKind;
  operation: string;
  status: number;
  body: unknown;
  requestId?: string | null;
}

export interface SkillSpec {
  id: string;
  name: string;
  description: string;
  provider: ByokProviderKind;
  capability: ProviderCapability;
  tags: string[];
  inputKeys: string[];
  outputKeys: string[];
  parallelizable: boolean;
}

export interface SkillRegistryEntry {
  spec: SkillSpec;
  enabled: boolean;
  weight?: number;
}

export interface TaskSpec {
  id: string;
  kind: TaskKind;
  skillId?: string | null;
  provider?: ByokProviderKind | null;
  input: Record<string, unknown>;
  dependsOn: string[];
}

export interface TaskExecutionOutcome {
  taskId: string;
  kind: TaskKind;
  provider?: ByokProviderKind | null;
  success: boolean;
  summary: string;
  payload: unknown;
  notes: string[];
  evidence: Array<{
    id: string;
    objective: string;
    taskId: string;
    claim: string;
    source: string;
    provider: string;
    confidence: number;
    contradictions: string[];
    notes: string[];
    payload: unknown;
    generatedAt: string;
  }>;
}

export interface WavePlan {
  id: string;
  taskIds: string[];
}

export interface OrchestratorContract {
  objective: string;
  tasks: TaskSpec[];
  gaps: string[];
  nextQueries: string[];
  maxWaves: number;
}

export interface ResearchClaimV2 {
  id: string;
  claim: string;
  confidence: number;
  sources: string[];
  contradictions: string[];
}

export interface OrchestrationReport {
  objective: string;
  summary: string;
  claims: ResearchClaimV2[];
  gaps: string[];
  nextQueries: string[];
  confidenceScore: number;
  generatedAt: string;
}

export interface EvidenceRecord {
  id: string;
  objective: string;
  taskId: string;
  claim: string;
  source: string;
  provider: string;
  confidence: number;
  contradictions: string[];
  notes: string[];
  payload: unknown;
  generatedAt: string;
}

export interface ProviderRoute {
  provider: ByokProviderKind;
  capability: ProviderCapability;
  target: string;
  operation: string;
}
