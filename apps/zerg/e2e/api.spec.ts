import { test, expect } from '@playwright/test';

test.describe('API Endpoints', () => {
  test('should return tasks from /tasks endpoint', async ({ request }) => {
    const response = await request.get('http://localhost:9080/tasks');
    expect(response.ok()).toBeTruthy();
    
    const tasks = await response.json();
    expect(Array.isArray(tasks)).toBeTruthy();
    expect(tasks.length).toBeGreaterThan(0);
    expect(tasks[0]).toHaveProperty('id');
    expect(tasks[0]).toHaveProperty('title');
    expect(tasks[0]).toHaveProperty('description');
    expect(tasks[0]).toHaveProperty('completed');
  });

  test('should handle chat requests to /api/chat', async ({ request }) => {
    const response = await request.post('http://localhost:9080/api/chat', {
      data: {
        message: 'Hello, test message'
      }
    });
    
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data).toHaveProperty('response');
    expect(typeof data.response).toBe('string');
  });

  test('should return metrics from /metrics endpoint', async ({ request }) => {
    const response = await request.get('http://localhost:9080/metrics');
    expect(response.ok()).toBeTruthy();
    
    const metricsText = await response.text();
    expect(metricsText).toContain('http_requests_total');
    expect(metricsText).toContain('process_resident_memory_bytes');
  });

  test('should handle streaming chat requests', async ({ request }) => {
    const response = await request.post('http://localhost:9080/api/chat/stream', {
      data: {
        message: 'Hello streaming test'
      }
    });
    
    expect(response.ok()).toBeTruthy();
    expect(response.headers()['content-type']).toContain('text/event-stream');
  });
});

test.describe('LangGraph Agent API', () => {
  test('should respond to chat requests', async ({ request }) => {
    const response = await request.post('http://localhost:3001/chat', {
      data: {
        message: 'Hello agent'
      }
    });
    
    expect(response.ok()).toBeTruthy();
    const data = await response.json();
    expect(data).toHaveProperty('response');
  });
});