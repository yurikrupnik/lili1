export interface SupervisorState {
  messages: any[];
  currentTask?: string;
  agentResponses: Record<string, any>;
  routingDecision?: 'KNOWLEDGE_BASE' | 'DEVELOPMENT' | 'BOTH' | 'HTTP_CALL' | 'COMPLETE';
  httpEndpoint?: string;
  taskComplete: boolean;
}

export interface AgentConfig {
  name: string;
  description: string;
  tools: any[];
  endpoint?: string;
  model?: string;
  provider?: 'openai' | 'anthropic' | 'google-genai';
}

export interface HttpAgentRequest {
  endpoint: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE';
  payload?: any;
  headers?: Record<string, string>;
  streaming?: boolean;
}

export interface SupervisorConfig {
  model: string;
  temperature?: number;
  provider?: 'openai' | 'anthropic' | 'bedrock';
  knowledgeAgent: AgentConfig;
  developmentAgent: AgentConfig;
  httpAgents?: Record<string, AgentConfig>;
}

export type AgentType = 'KNOWLEDGE_BASE' | 'DEVELOPMENT' | 'HTTP_AGENT';

export interface TaskAssignment {
  agent: AgentType;
  task: string;
  context: string;
  requirements: string;
  endpoint?: string;
}
