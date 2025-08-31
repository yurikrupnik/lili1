import { test, expect } from '@playwright/test';

test.describe('Basic Functionality Tests', () => {
  test('should load the main page', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');

    await expect(page.locator('h1')).toContainText('AI Agent Chat');
    await expect(page.locator('h2')).toContainText('Regular HTTP Chat');

    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    await expect(textarea).toBeVisible();

    const sendButton = page.locator('button', { hasText: 'Send' });
    await expect(sendButton).toBeVisible();
    await expect(sendButton).toBeDisabled();
  });

  test('should switch to streaming chat', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');

    const streamingButton = page.locator('button', { hasText: 'Streaming Chat' });
    await streamingButton.click();

    await expect(page.locator('h2')).toContainText('Streaming Chat');
  });

  test('should enable send button when text is entered', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForLoadState('networkidle');

    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    const sendButton = page.locator('button', { hasText: 'Send' });

    await expect(sendButton).toBeDisabled();

    await textarea.fill('Hello world');
    await expect(sendButton).toBeEnabled();

    await textarea.clear();
    await expect(sendButton).toBeDisabled();
  });

  test('should return tasks from API', async ({ request }) => {
    const response = await request.get('http://localhost:9080/tasks');
    expect(response.ok()).toBeTruthy();

    const tasks = await response.json();
    expect(Array.isArray(tasks)).toBeTruthy();
    expect(tasks.length).toBeGreaterThan(0);
  });

  test('should return metrics from API', async ({ request }) => {
    const response = await request.get('http://localhost:9080/metrics');
    expect(response.ok()).toBeTruthy();

    const metricsText = await response.text();
    expect(metricsText).toContain('http_requests_total');
  });
});
