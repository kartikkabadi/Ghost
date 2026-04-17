export type ProviderKind = "openRouter" | "xai" | "firecrawl" | "exa";
export type ProviderCapability = "plan" | "reason" | "search" | "crawl" | "extract" | "enrich" | "report";
export type TaskKind = "plan" | "search" | "crawl" | "extract" | "enrich" | "verify" | "synthesize" | "report" | { custom: string };

export interface GatewayRequest {
  provider: ProviderKind;
  operation: string;
  method: string;
  url: string;
  headers: Record<string, string>;
  body: unknown;
}

export interface GatewayResponse {
  provider: ProviderKind;
  operation: string;
  status: number;
  body: unknown;
  requestId?: string | null;
}

export interface SkillSpec {
  id: string;
  name: string;
  description: string;
  provider: ProviderKind;
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
  provider?: ProviderKind | null;
  input: Record<string, unknown>;
  dependsOn: string[];
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

export interface TaskExecutionOutcome {
  taskId: string;
  kind: TaskKind;
  provider?: ProviderKind | null;
  success: boolean;
  summary: string;
  payload: unknown;
  notes: string[];
  evidence: EvidenceRecord[];
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

export interface ResearchClaim {
  id: string;
  claim: string;
  confidence: number;
  sources: string[];
  contradictions: string[];
}

export interface ResearchReport {
  objective: string;
  summary: string;
  claims: ResearchClaim[];
  gaps: string[];
  nextQueries: string[];
  confidenceScore: number;
  generatedAt: string;
}

export type ResearchClaimV2 = ResearchClaim;
export type EvidenceClaim = ResearchClaim;
export type ByokProviderKind = ProviderKind;

export interface ProviderRoute {
  provider: ProviderKind;
  capability: ProviderCapability;
  target: string;
  operation: string;
}
