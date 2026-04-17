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
