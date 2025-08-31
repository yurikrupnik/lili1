import { ChatGoogleGenerativeAI } from "@langchain/google-genai";
import { createReactAgent } from "@langchain/langgraph/prebuilt";
import { KNOWLEDGE_BASE_TOOLS } from "./tools.js";

const KNOWLEDGE_BASE_SYSTEM_PROMPT = `You are a Knowledge Base Agent specializing in information retrieval, research, and knowledge management.

Your capabilities include:
- Searching through documentation and knowledge bases
- Retrieving specific information on topics
- Analyzing content for insights and summaries
- Providing research-backed answers
- Organizing and structuring information

Always provide accurate, well-researched information. When you don't know something, clearly state that and suggest how to find the information.

Focus on being thorough in your research and clear in your explanations.`;

export interface KnowledgeBaseConfig {
  modelName?: string;
  temperature?: number;
}

export interface IKnowledgeBaseAgent {
  invoke: (messages: any[], config?: any) => Promise<any>;
  stream: (messages: any[], config?: any) => Promise<any>;
}

const createModel = (config: KnowledgeBaseConfig) =>  {
  return new ChatGoogleGenerativeAI({
    model: config.modelName ?? "gemini-1.5-pro",
    temperature: config.temperature ?? 0.1,
  });
}

const createAgent = (model: ChatGoogleGenerativeAI) =>
  createReactAgent({
    llm: model,
    // stateSchema
    // preModelHook: {
      // messages :"",
      // llmInputMessages: ""
    // },
    tools: KNOWLEDGE_BASE_TOOLS,
  });

const createSystemMessage = () => ({
  role: "system" as const,
  content: KNOWLEDGE_BASE_SYSTEM_PROMPT,
});

const invokeAgent = (agent: any) => async (messages: any[], config?: any) => {
  const systemMessage = createSystemMessage();
  return await agent.invoke({
    messages: [systemMessage, ...messages],
  }, config);
};

const streamAgent = (agent: any) => async (messages: any[], config?: any) => {
  const systemMessage = createSystemMessage();
  return agent.stream({
    messages: [systemMessage, ...messages],
  }, config);
};

export const createKnowledgeBaseAgent = (config: KnowledgeBaseConfig = {}): IKnowledgeBaseAgent => {
  const model = createModel(config);
  const agent = createAgent(model);

  return {
    invoke: invokeAgent(agent),
    stream: streamAgent(agent),
  };
};

// Legacy class export for backward compatibility
export class KnowledgeBaseAgent {
  private agent: ReturnType<typeof createKnowledgeBaseAgent>;

  constructor(modelName: string = "gemini-1.5-pro") {
    this.agent = createKnowledgeBaseAgent({ modelName });
  }

  async invoke(messages: any[], config?: any) {
    return this.agent.invoke(messages, config);
  }

  async stream(messages: any[], config?: any) {
    return this.agent.stream(messages, config);
  }
}
