import { ChatOpenAI } from "@langchain/openai";
import { AIMessage } from "@langchain/core/messages";
import { MessagesAnnotation, StateGraph } from "@langchain/langgraph";
import { ToolNode } from "@langchain/langgraph/prebuilt";
import { SUPERVISOR_TOOLS } from "./tools.js";
import { SUPERVISOR_SYSTEM_PROMPT } from "./prompts.js";

const model = new ChatOpenAI({
  model: "gpt-4o",
  temperature: 0.3,
});

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

function routeSupervisorOutput(state: typeof MessagesAnnotation.State): string {
  const lastMessage = state.messages[state.messages.length - 1];
  const toolCalls = (lastMessage as AIMessage)?.tool_calls || [];

  if (toolCalls.length > 0) {
    const toolCall = toolCalls[0];
    
    switch (toolCall.name) {
      case "route_to_knowledge_base":
      case "route_to_development": 
      case "route_to_both_agents":
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
  .addEdge("__start__", "supervisor")
  .addConditionalEdges("supervisor", routeSupervisorOutput)
  .addEdge("tools", "supervisor");

export const graph = workflow.compile();