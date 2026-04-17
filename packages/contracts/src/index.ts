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
  verificationStatus?: "verified" | "conflicted" | "unverified" | null;
  evidenceCount: number;
}

export interface ResearchBlindSpot {
  id: string;
  gap: string;
  impact: string;
  unmetConstraints: string[];
  suggestedFollowUpQueries: string[];
  relatedClaims: string[];
}

export interface ResearchReport {
  objective: string;
  summary: string;
  claims: ResearchClaim[];
  blindSpots: ResearchBlindSpot[];
  gaps: string[];
  nextQueries: string[];
  confidenceScore: number;
  generatedAt: string;
  markdownBrief: string;
  jsonBrief: unknown;
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


export type AgentMode = 4 | 16;

export interface ModelProfile {
  provider: ProviderKind;
  model: string;
  baseUrl: string;
  sdkPattern: 'chat-completions' | 'multi-agent' | 'search';
  agentModes: AgentMode[];
  specialization?: string;
  notes?: string[];
}

export interface OsintPrimitive {
  id: string;
  title: string;
  description: string;
  inputs: string[];
  outputs: string[];
  verificationChecks: string[];
}

export interface RuntimeConfig {
  version: string;
  providers: Record<ProviderKind, { baseUrl: string; defaultOperation: string }>;
  modelProfiles: ModelProfile[];
  osintPrimitives: OsintPrimitive[];
}