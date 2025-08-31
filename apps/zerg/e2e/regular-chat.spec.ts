import { test, expect } from '@playwright/test';

test.describe('Regular Chat Interface', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display the regular chat interface by default', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('AI Agent Chat');
    await expect(page.locator('h2')).toContainText('Regular HTTP Chat');
    
    const regularButton = page.locator('button', { hasText: 'Regular Chat' });
    await expect(regularButton).toBeVisible();
    
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    await expect(textarea).toBeVisible();
    
    const sendButton = page.locator('button', { hasText: 'Send' });
    await expect(sendButton).toBeVisible();
    await expect(sendButton).toBeDisabled();
  });

  test('should enable send button when text is entered', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    const sendButton = page.locator('button', { hasText: 'Send' });
    
    await expect(sendButton).toBeDisabled();
    
    await textarea.fill('Hello world');
    await expect(sendButton).toBeEnabled();
    
    await textarea.clear();
    await expect(sendButton).toBeDisabled();
  });

  test('should send message and display response', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    const sendButton = page.locator('button', { hasText: 'Send' });
    const chatArea = page.locator('[style*="height: 400px"]');
    
    const testMessage = 'Hello, this is a test message';
    await textarea.fill(testMessage);
    await sendButton.click();
    
    await expect(page.locator('text=Sending...')).toBeVisible();
    
    await expect(page.locator(`text=${testMessage}`)).toBeVisible();
    await expect(page.locator('text=You')).toBeVisible();
    
    await expect(page.locator('text=Sending...')).not.toBeVisible({ timeout: 10000 });
    
    await expect(page.locator('text=AI Assistant')).toBeVisible();
    
    const responseElements = page.locator('div').filter({ hasText: /AI Assistant/ });
    await expect(responseElements).toHaveCount(1);
    
    await expect(textarea).toHaveValue('');
  });

  test('should handle Enter key to send messages', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    
    const testMessage = 'Testing Enter key functionality';
    await textarea.fill(testMessage);
    await textarea.press('Enter');
    
    await expect(page.locator('text=Sending...')).toBeVisible();
    await expect(page.locator(`text=${testMessage}`)).toBeVisible();
  });

  test('should not send empty messages', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    const sendButton = page.locator('button', { hasText: 'Send' });
    
    await textarea.fill('   ');
    await expect(sendButton).toBeDisabled();
    
    await textarea.press('Enter');
    await expect(page.locator('text=Sending...')).not.toBeVisible();
  });

  test('should display timestamps for messages', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    const sendButton = page.locator('button', { hasText: 'Send' });
    
    await textarea.fill('Test timestamp');
    await sendButton.click();
    
    await expect(page.locator('text=Sending...')).toBeVisible();
    await expect(page.locator('text=Sending...')).not.toBeVisible({ timeout: 10000 });
    
    const timePattern = /\d{1,2}:\d{2}:\d{2}/;
    await expect(page.locator(`text=${timePattern.source}`).first()).toBeVisible();
  });

  test('should handle multiple messages in sequence', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    const sendButton = page.locator('button', { hasText: 'Send' });
    
    const messages = ['First message', 'Second message', 'Third message'];
    
    for (const message of messages) {
      await textarea.fill(message);
      await sendButton.click();
      await expect(page.locator('text=Sending...')).not.toBeVisible({ timeout: 10000 });
    }
    
    for (const message of messages) {
      await expect(page.locator(`text=${message}`)).toBeVisible();
    }
    
    const userMessages = page.locator('text=You');
    await expect(userMessages).toHaveCount(3);
  });
});