import { createReactAgent } from "@langchain/langgraph/prebuilt";
// import { TOOLS } from "./tools.js";
import { loadChatModel } from "./utils.js";

// Create the agent using the prebuilt createReactAgent function
const model = await loadChatModel("claude-3-5-sonnet-20240620");
export const graph = createReactAgent({
  llm: model,
  tools: [],
});
