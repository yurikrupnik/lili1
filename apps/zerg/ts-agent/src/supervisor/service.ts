import { SupervisorGraph } from "./graph.js";
import { SupervisorConfig } from "./types.js";

export class SupervisorService {
  private graph: SupervisorGraph;
  private config: SupervisorConfig;

  constructor(config: SupervisorConfig) {
    this.config = config;
    this.graph = new SupervisorGraph(config);
  }

  async processRequest(messages: any[], streaming: boolean = false) {
    try {
      if (streaming) {
        return await this.graph.stream(messages);
      } else {
        return await this.graph.invoke(messages);
      }
    } catch (error) {
      throw new Error(`Supervisor service error: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  async handleUserMessage(userMessage: string, streaming: boolean = false) {
    const messages = [
      {
        role: "user" as const,
        content: userMessage,
      },
    ];

    return await this.processRequest(messages, streaming);
  }

  getConfig(): SupervisorConfig {
    return this.config;
  }

  updateConfig(newConfig: Partial<SupervisorConfig>) {
    this.config = { ...this.config, ...newConfig };
    this.graph = new SupervisorGraph(this.config);
  }
}

export function createSupervisorService(config?: Partial<SupervisorConfig>): SupervisorService {
  const defaultConfig: SupervisorConfig = {
    model: "gpt-4o",
    temperature: 0.3,
    provider: "openai",
    knowledgeAgent: {
      name: "knowledge_base",
      description: "Expert in information retrieval, research, and knowledge management",
      tools: [],
      model: "gemini-1.5-pro",
      provider: "google-genai",
    },
    developmentAgent: {
      name: "development",
      description: "Expert in software development, coding, debugging, and technical implementation",
      tools: [],
      model: "gpt-4o",
      provider: "openai",
    },
  };

  const finalConfig = { ...defaultConfig, ...config };
  return new SupervisorService(finalConfig);
}