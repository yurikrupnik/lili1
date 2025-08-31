import { DynamicStructuredTool } from "@langchain/core/tools";
import { z } from "zod";
import { HttpAgentClient } from "./http-client.js";
import { createKnowledgeBaseAgent } from "../agents/knowledge-base/agent.js";
import { DevelopmentAgent } from "../agents/development/agent.js";

const httpClient = new HttpAgentClient();
const knowledgeAgent = createKnowledgeBaseAgent({ modelName: "gemini-1.5-pro" });
const developmentAgent = new DevelopmentAgent("gpt-4o");

export const routeToKnowledgeBase = new DynamicStructuredTool({
  name: "route_to_knowledge_base",
  description: "Route a task to the knowledge base agent",
  schema: z.object({
    task: z.string().describe("The task to assign to the knowledge base agent"),
    context: z.string().describe("Additional context for the task"),
    requirements: z.string().describe("Specific requirements or constraints"),
  }),
  func: async ({ task, context, requirements }) => {
    try {
      const taskMessage = {
        role: "user" as const,
        content: `Task: ${task}\nContext: ${context}\nRequirements: ${requirements}`,
      };

      const result = await knowledgeAgent.invoke([taskMessage]);
      const content = result.messages[result.messages.length - 1].content;

      return `Knowledge Agent completed task successfully:\n${typeof content === 'string' ? content : JSON.stringify(content)}`;
    } catch (error) {
      return `Knowledge Agent error: ${error instanceof Error ? error.message : String(error)}`;
    }
  },
});

export const routeToDevelopment = new DynamicStructuredTool({
  name: "route_to_development",
  description: "Route a task to the development agent",
  schema: z.object({
    task: z.string().describe("The task to assign to the development agent"),
    context: z.string().describe("Additional context for the task"),
    requirements: z.string().describe("Specific requirements or constraints"),
  }),
  func: async ({ task, context, requirements }) => {
    try {
      const taskMessage = {
        role: "user" as const,
        content: `Task: ${task}\nContext: ${context}\nRequirements: ${requirements}`,
      };

      const result = await developmentAgent.invoke([taskMessage]);
      const content = result.messages[result.messages.length - 1].content;

      return `Development Agent completed task successfully:\n${typeof content === 'string' ? content : JSON.stringify(content)}`;
    } catch (error) {
      return `Development Agent error: ${error instanceof Error ? error.message : String(error)}`;
    }
  },
});

export const routeToBothAgents = new DynamicStructuredTool({
  name: "route_to_both_agents",
  description: "Route a task to both knowledge base and development agents",
  schema: z.object({
    knowledgeTask: z.string().describe("Task for the knowledge base agent"),
    developmentTask: z.string().describe("Task for the development agent"),
    context: z.string().describe("Shared context for both agents"),
    coordination: z.string().describe("How the agents should coordinate"),
  }),
  func: async ({ knowledgeTask, developmentTask, context, coordination }) => {
    try {
      const knowledgeMessage = {
        role: "user" as const,
        content: `Task: ${knowledgeTask}\nContext: ${context}\nCoordination: ${coordination}`,
      };

      const developmentMessage = {
        role: "user" as const,
        content: `Task: ${developmentTask}\nContext: ${context}\nCoordination: ${coordination}`,
      };

      const [knowledgeResult, developmentResult] = await Promise.all([
        knowledgeAgent.invoke([knowledgeMessage]),
        developmentAgent.invoke([developmentMessage])
      ]);

      const knowledgeContent = knowledgeResult.messages[knowledgeResult.messages.length - 1].content;
      const developmentContent = developmentResult.messages[developmentResult.messages.length - 1].content;

      return `Both agents completed their tasks:\n\nKnowledge Agent:\n${typeof knowledgeContent === 'string' ? knowledgeContent : JSON.stringify(knowledgeContent)}\n\nDevelopment Agent:\n${typeof developmentContent === 'string' ? developmentContent : JSON.stringify(developmentContent)}`;
    } catch (error) {
      return `Error coordinating both agents: ${error instanceof Error ? error.message : String(error)}`;
    }
  },
});

export const callHttpAgent = new DynamicStructuredTool({
  name: "call_http_agent",
  description: "Make an HTTP call to an external agent service",
  schema: z.object({
    endpoint: z.string().describe("The HTTP endpoint URL"),
    method: z.enum(["GET", "POST", "PUT", "DELETE"]).describe("HTTP method"),
    payload: z.any().optional().describe("Request payload"),
    streaming: z.boolean().optional().describe("Whether to use streaming"),
    agentSpecialty: z.string().describe("What the external agent specializes in"),
  }),
  func: async ({ endpoint, method, payload, streaming = false, agentSpecialty }) => {
    try {
      const result = await httpClient.callAgent({
        endpoint,
        method,
        payload,
        streaming,
      });
      
      return JSON.stringify({
        action: "HTTP_CALL_RESULT",
        endpoint,
        agentSpecialty,
        result,
        success: true,
      });
    } catch (error) {
      return JSON.stringify({
        action: "HTTP_CALL_ERROR",
        endpoint,
        agentSpecialty,
        error: error instanceof Error ? error.message : String(error),
        success: false,
      });
    }
  },
});

export const completeTask = new DynamicStructuredTool({
  name: "complete_task",
  description: "Mark the current task as complete and provide final response",
  schema: z.object({
    summary: z.string().describe("Summary of work completed"),
    finalResponse: z.string().describe("Final response to the user"),
    agentsUsed: z.array(z.string()).describe("List of agents that contributed"),
  }),
  func: async ({ summary, finalResponse, agentsUsed }) => {
    return JSON.stringify({
      action: "COMPLETE",
      summary,
      finalResponse,
      agentsUsed,
    });
  },
});

export const SUPERVISOR_TOOLS = [
  routeToKnowledgeBase,
  routeToDevelopment,
  routeToBothAgents,
  callHttpAgent,
  completeTask,
];