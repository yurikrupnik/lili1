import { ChatOpenAI } from "@langchain/openai";
import { AIMessage } from "@langchain/core/messages";
import { RunnableConfig } from "@langchain/core/runnables";
import { MessagesAnnotation, StateGraph } from "@langchain/langgraph";
import { ToolNode } from "@langchain/langgraph/prebuilt";
import { SUPERVISOR_TOOLS } from "./tools.js";
import { SUPERVISOR_SYSTEM_PROMPT } from "./prompts.js";
import { SupervisorConfig } from "./types.js";

export class SupervisorGraph {
  private model: ChatOpenAI;
  private graph: any;

  constructor(config: SupervisorConfig) {
    this.model = new ChatOpenAI({
      model: config.model,
      temperature: config.temperature || 0.3,
    });

    this.graph = this.buildGraph();
  }

  private buildGraph() {
    const workflow = new StateGraph(MessagesAnnotation)
      .addNode("supervisor", this.callSupervisor.bind(this))
      .addNode("tools", new ToolNode(SUPERVISOR_TOOLS))
      .addEdge("__start__", "supervisor")
      .addConditionalEdges(
        "supervisor",
        this.routeSupervisorOutput.bind(this)
      )
      .addEdge("tools", "supervisor");

    return workflow.compile();
  }

  private async callSupervisor(
    state: typeof MessagesAnnotation.State
  ): Promise<typeof MessagesAnnotation.Update> {
    const systemPrompt = SUPERVISOR_SYSTEM_PROMPT.replace(
      "{system_time}",
      new Date().toISOString()
    );

    const modelWithTools = this.model.bindTools(SUPERVISOR_TOOLS);

    const response = await modelWithTools.invoke([
      { role: "system", content: systemPrompt },
      ...state.messages,
    ]);

    return { messages: [response] };
  }


  private routeSupervisorOutput(state: typeof MessagesAnnotation.State): string {
    const lastMessage = state.messages[state.messages.length - 1];
    const toolCalls = (lastMessage as AIMessage)?.tool_calls || [];

    if (toolCalls.length > 0) {
      return "tools";
    }

    return "__end__";
  }

  async invoke(messages: any[], config?: RunnableConfig) {
    return await this.graph.invoke({ messages }, config);
  }

  async stream(messages: any[], config?: RunnableConfig) {
    return this.graph.stream({ messages }, config);
  }

  getGraph() {
    return this.graph;
  }
}

// Export a default instance for LangGraph Studio
const defaultConfig: SupervisorConfig = {
  model: "gpt-4o",
  temperature: 0.3,
  provider: "openai",
  knowledgeAgent: {
    name: "knowledge_base",
    description: "Expert in information retrieval, research, and knowledge management",
    tools: [],
  },
  developmentAgent: {
    name: "development",
    description: "Expert in software development, coding, debugging, and technical implementation",
    tools: [],
  },
};

const supervisorGraph = new SupervisorGraph(defaultConfig);
export const graph = supervisorGraph.getGraph();
