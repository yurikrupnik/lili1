import { ChatOpenAI } from "@langchain/openai";
import { AIMessage } from "@langchain/core/messages";
import { MessagesAnnotation, StateGraph } from "@langchain/langgraph";
import { ToolNode } from "@langchain/langgraph/prebuilt";
import { SUPERVISOR_TOOLS } from "./tools.js";
import { SUPERVISOR_SYSTEM_PROMPT } from "./prompts.js";
import { KnowledgeBaseAgent } from "../agents/knowledge-base/agent.js";
import { DevelopmentAgent } from "../agents/development/agent.js";

const model = new ChatOpenAI({
  model: "gpt-4o",
  temperature: 0.3,
});

const knowledgeAgent = new KnowledgeBaseAgent("gemini-1.5-pro");
const developmentAgent = new DevelopmentAgent("gpt-4o");

async function callSupervisor(
  state: typeof MessagesAnnotation.State
): Promise<typeof MessagesAnnotation.Update> {
  const systemPrompt = SUPERVISOR_SYSTEM_PROMPT.replace(
    "{system_time}",
    new Date().toISOString()
  );

  const modelWithTools = model.bindTools(SUPERVISOR_TOOLS);
  
  const response = await modelWithTools.invoke([
    { role: "system", content: systemPrompt },
    ...state.messages,
  ]);

  return { messages: [response] };
}

async function callKnowledgeAgent(
  state: typeof MessagesAnnotation.State
): Promise<typeof MessagesAnnotation.Update> {
  const lastMessage = state.messages[state.messages.length - 1];
  
  try {
    const toolCall = (lastMessage as AIMessage).tool_calls?.[0];
    if (toolCall && toolCall.name === "route_to_knowledge_base") {
      const { task, context, requirements } = toolCall.args;
      
      const taskMessage = {
        role: "user" as const,
        content: `Task: ${task}\nContext: ${context}\nRequirements: ${requirements}`,
      };

      const result = await knowledgeAgent.invoke([taskMessage]);
      
      return {
        messages: [{
          role: "assistant" as const,
          content: `Knowledge Agent Result: ${result.messages[result.messages.length - 1].content}`,
        }],
      };
    }
  } catch (error) {
    return {
      messages: [{
        role: "assistant" as const,
        content: `Knowledge Agent Error: ${error instanceof Error ? error.message : String(error)}`,
      }],
    };
  }

  return { messages: [] };
}

async function callDevelopmentAgent(
  state: typeof MessagesAnnotation.State
): Promise<typeof MessagesAnnotation.Update> {
  const lastMessage = state.messages[state.messages.length - 1];
  
  try {
    const toolCall = (lastMessage as AIMessage).tool_calls?.[0];
    if (toolCall && toolCall.name === "route_to_development") {
      const { task, context, requirements } = toolCall.args;
      
      const taskMessage = {
        role: "user" as const,
        content: `Task: ${task}\nContext: ${context}\nRequirements: ${requirements}`,
      };

      const result = await developmentAgent.invoke([taskMessage]);
      
      return {
        messages: [{
          role: "assistant" as const,
          content: `Development Agent Result: ${result.messages[result.messages.length - 1].content}`,
        }],
      };
    }
  } catch (error) {
    return {
      messages: [{
        role: "assistant" as const,
        content: `Development Agent Error: ${error instanceof Error ? error.message : String(error)}`,
      }],
    };
  }

  return { messages: [] };
}

async function callBothAgents(
  state: typeof MessagesAnnotation.State
): Promise<typeof MessagesAnnotation.Update> {
  const lastMessage = state.messages[state.messages.length - 1];
  
  try {
    const toolCall = (lastMessage as AIMessage).tool_calls?.[0];
    if (toolCall && toolCall.name === "route_to_both_agents") {
      const { knowledgeTask, developmentTask, context } = toolCall.args;
      
      // Call both agents in parallel
      const [knowledgeResult, developmentResult] = await Promise.all([
        knowledgeAgent.invoke([{
          role: "user" as const,
          content: `Task: ${knowledgeTask}\nContext: ${context}`,
        }]),
        developmentAgent.invoke([{
          role: "user" as const,
          content: `Task: ${developmentTask}\nContext: ${context}`,
        }])
      ]);
      
      return {
        messages: [{
          role: "assistant" as const,
          content: `Both Agents Results:
Knowledge Agent: ${knowledgeResult.messages[knowledgeResult.messages.length - 1].content}

Development Agent: ${developmentResult.messages[developmentResult.messages.length - 1].content}`,
        }],
      };
    }
  } catch (error) {
    return {
      messages: [{
        role: "assistant" as const,
        content: `Both Agents Error: ${error instanceof Error ? error.message : String(error)}`,
      }],
    };
  }

  return { messages: [] };
}

function routeSupervisorOutput(state: typeof MessagesAnnotation.State): string {
  const lastMessage = state.messages[state.messages.length - 1];
  const toolCalls = (lastMessage as AIMessage)?.tool_calls || [];

  if (toolCalls.length > 0) {
    const toolCall = toolCalls[0];
    
    switch (toolCall.name) {
      case "route_to_knowledge_base":
        return "knowledge_agent";
      case "route_to_development":
        return "development_agent";
      case "route_to_both_agents":
        return "both_agents";
      case "call_http_agent":
      case "complete_task":
        return "tools";
      default:
        return "tools";
    }
  }

  return "__end__";
}

const workflow = new StateGraph(MessagesAnnotation)
  .addNode("supervisor", callSupervisor)
  .addNode("tools", new ToolNode(SUPERVISOR_TOOLS))
  .addNode("knowledge_agent", callKnowledgeAgent)
  .addNode("development_agent", callDevelopmentAgent)
  .addNode("both_agents", callBothAgents)
  .addEdge("__start__", "supervisor")
  .addConditionalEdges("supervisor", routeSupervisorOutput)
  .addEdge("tools", "supervisor")
  .addEdge("knowledge_agent", "supervisor")
  .addEdge("development_agent", "supervisor")
  .addEdge("both_agents", "supervisor");

export const graph = workflow.compile();