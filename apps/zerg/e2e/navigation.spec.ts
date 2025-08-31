import { test, expect } from '@playwright/test';

test.describe('Navigation and UI Layout', () => {
  test('should load the main page with correct title and navigation', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    await expect(page).toHaveTitle('Rsbuild App');
    
    await expect(page.locator('h1')).toContainText('AI Agent Chat');
    
    await expect(page.locator('button', { hasText: 'Regular Chat' })).toBeVisible();
    await expect(page.locator('button', { hasText: 'Streaming Chat' })).toBeVisible();
    
    const nav = page.locator('nav');
    await expect(nav).toBeVisible();
    await expect(nav).toContainText('AI Agent Chat');
  });

  test('should have proper responsive layout', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const container = page.locator('[style*="max-width: 800px"]');
    await expect(container).toBeVisible();
    
    const chatArea = page.locator('[style*="height: 400px"]');
    await expect(chatArea).toBeVisible();
  });

  test('should maintain state when switching between chat modes', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    await textarea.fill('Test state persistence');
    
    const streamingButton = page.locator('button', { hasText: 'Streaming Chat' });
    await streamingButton.click();
    
    const newTextarea = page.locator('textarea[placeholder="Type your message here..."]');
    await expect(newTextarea).toHaveValue('');
    
    const regularButton = page.locator('button', { hasText: 'Regular Chat' });
    await regularButton.click();
    
    const backToRegularTextarea = page.locator('textarea[placeholder="Type your message here..."]');
    await expect(backToRegularTextarea).toHaveValue('');
  });

  test('should have accessible form elements', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const textarea = page.locator('textarea[placeholder="Type your message here..."]');
    await expect(textarea).toBeVisible();
    await expect(textarea).not.toBeDisabled();
    
    const sendButton = page.locator('button', { hasText: 'Send' });
    await expect(sendButton).toBeVisible();
    
    await textarea.fill('test');
    await expect(sendButton).not.toBeDisabled();
  });

  test('should handle page refresh gracefully', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    await page.reload();
    await page.waitForLoadState('networkidle');
    
    await expect(page.locator('h1')).toContainText('AI Agent Chat');
    await expect(page.locator('h2')).toContainText('Regular HTTP Chat');
    
    const regularButton = page.locator('button', { hasText: 'Regular Chat' });
    await expect(regularButton).toHaveAttribute('variant', /default/);
  });
});