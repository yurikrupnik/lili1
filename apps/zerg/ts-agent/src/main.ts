import { createSupervisorService } from "./supervisor/service.js";
import { SupervisorConfig } from "./supervisor/types.js";

async function main() {
  const config: Partial<SupervisorConfig> = {
    model: "gpt-4o",
    temperature: 0.3,
    provider: "openai",
    httpAgents: {
      "external_research": {
        name: "external_research",
        description: "External research agent for web scraping and data analysis",
        tools: [],
        endpoint: "http://localhost:8080/research",
      },
      "code_review": {
        name: "code_review",
        description: "External code review agent for advanced static analysis",
        tools: [],
        endpoint: "http://localhost:8081/review",
      },
    },
  };

  const supervisorService = createSupervisorService(config);

  console.log("Supervisor Agent Service Started");
  console.log("Available agents:");
  console.log("- Knowledge Base Agent (Gemini 1.5 Pro): Research, documentation, information retrieval");
  console.log("- Development Agent (GPT-4o): Code generation, review, debugging, optimization");
  console.log("- HTTP Agents: External services for specialized tasks");
  console.log("");

  const examples = [
    "Research the latest TypeScript features and create a migration guide",
    "Help me debug this React performance issue and optimize the component",
    "Find best practices for microservices architecture and implement a sample service",
  ];

  console.log("Example requests:");
  examples.forEach((example, index) => {
    console.log(`${index + 1}. ${example}`);
  });

  const testMessage = "I need help understanding LangGraph architecture and implementing a multi-agent system";
  
  console.log(`\nTesting with: "${testMessage}"`);
  
  try {
    const result = await supervisorService.handleUserMessage(testMessage);
    console.log("\nResult:");
    console.log(JSON.stringify(result, null, 2));
  } catch (error) {
    console.error("Error:", error);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}

export { createSupervisorService } from "./supervisor/service.js";
export { SupervisorService } from "./supervisor/service.js";
export { SupervisorGraph } from "./supervisor/graph.js";
export { KnowledgeBaseAgent } from "./agents/knowledge-base/agent.js";
export { DevelopmentAgent } from "./agents/development/agent.js";
export * from "./supervisor/types.js";