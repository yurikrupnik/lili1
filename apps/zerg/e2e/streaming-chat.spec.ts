import { test, expect } from '@playwright/test';

test.describe('Streaming Chat Interface', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const streamingButton = page.locator('button', { hasText: 'Streaming Chat' });
    await streamingButton.click();
    await expect(page.locator('h2')).toContainText('Streaming Chat');
  });

  test('should switch to streaming chat interface', async ({ page }) => {
    await expect(page.locator('h2')).toContainText('Streaming Chat');
    
    const streamingButton = page.locator('button', { hasText: 'Streaming Chat' });
    await expect(streamingButton).toHaveAttribute('variant', /default/);
    
    const regularButton = page.locator('button', { hasText: 'Regular Chat' });
    await expect(regularButton).toHaveAttribute('variant', /secondary/);
    
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    await expect(textarea).toBeVisible();
    
    const sendButton = page.locator('button', { hasText: 'Send' });
    await expect(sendButton).toBeVisible();
    await expect(sendButton).toBeDisabled();
  });

  test('should enable send button when text is entered in streaming chat', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    const sendButton = page.locator('button', { hasText: 'Send' });
    
    await expect(sendButton).toBeDisabled();
    
    await textarea.fill('Hello streaming world');
    await expect(sendButton).toBeEnabled();
    
    await textarea.clear();
    await expect(sendButton).toBeDisabled();
  });

  test('should send message and show streaming response', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    const sendButton = page.locator('button', { hasText: 'Send' });
    
    const testMessage = 'Hello, this is a streaming test message';
    await textarea.fill(testMessage);
    await sendButton.click();
    
    await expect(page.locator('text=Stop')).toBeVisible();
    
    await expect(page.locator(`text=${testMessage}`)).toBeVisible();
    await expect(page.locator('text=You')).toBeVisible();
    
    await expect(page.locator('text=streaming...')).toBeVisible();
    
    await expect(page.locator('text=AI Assistant')).toBeVisible();
    
    await expect(page.locator('text=Stop')).not.toBeVisible({ timeout: 15000 });
    await expect(page.locator('text=streaming...')).not.toBeVisible({ timeout: 15000 });
    
    await expect(textarea).toHaveValue('');
  });

  test('should show stop button during streaming and allow stopping', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    const sendButton = page.locator('button', { hasText: 'Send' });
    
    await textarea.fill('Test stopping stream');
    await sendButton.click();
    
    const stopButton = page.locator('button', { hasText: 'Stop' });
    await expect(stopButton).toBeVisible();
    
    await stopButton.click();
    
    await expect(stopButton).not.toBeVisible();
    await expect(page.locator('button', { hasText: 'Send' })).toBeVisible();
    await expect(page.locator('text=streaming...')).not.toBeVisible();
  });

  test('should handle Enter key in streaming chat', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    
    const testMessage = 'Testing Enter key in streaming';
    await textarea.fill(testMessage);
    await textarea.press('Enter');
    
    await expect(page.locator('text=Stop')).toBeVisible();
    await expect(page.locator(`text=${testMessage}`)).toBeVisible();
  });

  test('should display streaming indicator and cursor animation', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    const sendButton = page.locator('button', { hasText: 'Send' });
    
    await textarea.fill('Test streaming animation');
    await sendButton.click();
    
    await expect(page.locator('text=streaming...')).toBeVisible();
    
    const cursorElement = page.locator('span').filter({ hasText: '▊' });
    await expect(cursorElement).toBeVisible();
  });

  test('should switch between regular and streaming interfaces', async ({ page }) => {
    const regularButton = page.locator('button', { hasText: 'Regular Chat' });
    const streamingButton = page.locator('button', { hasText: 'Streaming Chat' });
    
    await expect(page.locator('h2')).toContainText('Streaming Chat');
    
    await regularButton.click();
    await expect(page.locator('h2')).toContainText('Regular HTTP Chat');
    
    await streamingButton.click();
    await expect(page.locator('h2')).toContainText('Streaming Chat');
  });

  test('should display different colored messages for streaming chat', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    const sendButton = page.locator('button', { hasText: 'Send' });
    
    await textarea.fill('Test message colors');
    await sendButton.click();
    
    await expect(page.locator('text=Stop')).not.toBeVisible({ timeout: 15000 });
    
    const userMessage = page.locator('div').filter({ hasText: 'You' }).first();
    await expect(userMessage).toHaveCSS('background-color', /rgb\(232, 245, 232\)|#e8f5e8/);
    
    const assistantMessage = page.locator('div').filter({ hasText: 'AI Assistant' }).first();
    await expect(assistantMessage).toHaveCSS('background-color', /rgb\(255, 243, 205\)|#fff3cd/);
  });

  test('should handle multiple streaming messages', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    const sendButton = page.locator('button', { hasText: 'Send' });
    
    const messages = ['First streaming message', 'Second streaming message'];
    
    for (const message of messages) {
      await textarea.fill(message);
      await sendButton.click();
      await expect(page.locator('text=Stop')).not.toBeVisible({ timeout: 15000 });
    }
    
    for (const message of messages) {
      await expect(page.locator(`text=${message}`)).toBeVisible();
    }
    
    const userMessages = page.locator('text=You');
    await expect(userMessages).toHaveCount(2);
    
    const assistantMessages = page.locator('text=AI Assistant');
    await expect(assistantMessages).toHaveCount(2);
  });
});