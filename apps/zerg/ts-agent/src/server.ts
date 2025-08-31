import { createSupervisorService } from "./supervisor/service.js";
import { SupervisorConfig } from "./supervisor/types.js";
import { createServer } from "http";

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

const server = createServer(async (req, res) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');

  if (req.method === 'OPTIONS') {
    res.writeHead(200);
    res.end();
    return;
  }

  if (req.method === 'POST' && req.url === '/chat') {
    let body = '';
    req.on('data', chunk => {
      body += chunk.toString();
    });

    req.on('end', async () => {
      try {
        const { message, streaming } = JSON.parse(body);

        if (streaming) {
          res.setHeader('Content-Type', 'text/event-stream');
          res.setHeader('Cache-Control', 'no-cache');
          res.setHeader('Connection', 'keep-alive');

          const result = await supervisorService.handleUserMessage(message, true);
          
          for await (const chunk of result) {
            const chunkStr = typeof chunk === 'string' ? chunk : JSON.stringify(chunk);
            res.write(`data: ${JSON.stringify({ content: chunkStr, done: false })}\n\n`);
            
            await new Promise(resolve => setTimeout(resolve, 100));
          }
          
          res.write(`data: ${JSON.stringify({ content: '', done: true })}\n\n`);
          res.end();
        } else {
          const result = await supervisorService.handleUserMessage(message, false);
          const response = typeof result === 'string' ? result : JSON.stringify(result, null, 2);
          
          res.setHeader('Content-Type', 'application/json');
          res.writeHead(200);
          res.end(JSON.stringify({ response }));
        }
      } catch (error) {
        console.error('Error processing request:', error);
        res.setHeader('Content-Type', 'application/json');
        res.writeHead(500);
        res.end(JSON.stringify({ 
          error: `Internal server error: ${error instanceof Error ? error.message : String(error)}` 
        }));
      }
    });
  } else {
    res.writeHead(404);
    res.end('Not Found');
  }
});

const PORT = process.env.PORT || 3001;
server.listen(PORT, () => {
  console.log(`LangGraph Agent Server listening on port ${PORT}`);
  console.log(`Chat endpoint: http://localhost:${PORT}/chat`);
  console.log('Available agents:');
  console.log('- Knowledge Base Agent (Gemini 1.5 Pro): Research, documentation, information retrieval');
  console.log('- Development Agent (GPT-4o): Code generation, review, debugging, optimization');
});

export { server };