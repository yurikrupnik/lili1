import { ChatOpenAI } from "@langchain/openai";
import { createReactAgent } from "@langchain/langgraph/prebuilt";
import { DEVELOPMENT_TOOLS } from "./tools.js";

const DEVELOPMENT_SYSTEM_PROMPT = `You are a Development Agent specializing in software development, coding, debugging, and technical implementation.

Your capabilities include:
- Generating code in various programming languages
- Reviewing code for quality, security, and best practices
- Debugging code issues and providing solutions
- Optimizing code for performance and efficiency
- Providing technical guidance and implementation advice

Always follow software engineering best practices:
- Write clean, maintainable, and well-documented code
- Consider security implications in your solutions
- Follow language-specific conventions and patterns
- Suggest proper testing approaches
- Consider performance and scalability

Focus on providing practical, production-ready solutions.`;

export class DevelopmentAgent {
  private model: ChatOpenAI;
  private agent: any;

  constructor(modelName: string = "gpt-4o") {
    this.model = new ChatOpenAI({
      model: modelName,
      temperature: 0.2,
    });

    this.agent = createReactAgent({
      llm: this.model,
      tools: DEVELOPMENT_TOOLS,
    });
  }

  async invoke(messages: any[], config?: any) {
    const systemMessage = {
      role: "system" as const,
      content: DEVELOPMENT_SYSTEM_PROMPT,
    };

    return await this.agent.invoke({
      messages: [systemMessage, ...messages],
    }, config);
  }

  async stream(messages: any[], config?: any) {
    const systemMessage = {
      role: "system" as const,
      content: DEVELOPMENT_SYSTEM_PROMPT,
    };

    return this.agent.stream({
      messages: [systemMessage, ...messages],
    }, config);
  }
}