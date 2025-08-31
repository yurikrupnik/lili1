import { createSupervisorService } from "./supervisor/service.js";

async function testSupervisor() {
  console.log("🤖 Testing Supervisor Agent System");
  console.log("=" .repeat(50));

  const supervisorService = createSupervisorService({
    model: "gpt-4o",
    temperature: 0.3,
    provider: "openai",
  });

  const testCases = [
    {
      name: "Knowledge Request",
      message: "What are the best practices for TypeScript project structure?",
    },
    {
      name: "Development Request", 
      message: "Generate a TypeScript function to validate email addresses",
    },
    {
      name: "Mixed Request",
      message: "Research React performance optimization patterns and create a sample implementation",
    },
  ];

  for (const testCase of testCases) {
    console.log(`\n📝 Test Case: ${testCase.name}`);
    console.log(`💬 Message: "${testCase.message}"`);
    console.log("⏳ Processing...\n");

    try {
      const result = await supervisorService.handleUserMessage(testCase.message);
      console.log("✅ Result:", JSON.stringify(result.messages[result.messages.length - 1]?.content || "No content", null, 2));
    } catch (error) {
      console.error("❌ Error:", error);
    }

    console.log("\n" + "-".repeat(50));
  }

  console.log("\n🎉 Supervisor Agent System Test Complete!");
}

if (import.meta.url === `file://${process.argv[1]}`) {
  testSupervisor().catch(console.error);
}