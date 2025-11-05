import { test, expect } from '@playwright/test';

function toBase64Url(jsonString) {
  const b64 = Buffer.from(jsonString, 'utf-8').toString('base64');
  return b64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/g, '');
}

test.describe('Shareable URL State', () => {
  test('loads persistent elements from URL hash', async ({ page }) => {
    const logs = [];
    page.on('console', (msg) => logs.push({ type: msg.type(), text: msg.text() }));

    // Build a small share state: one dye source and one attractor
    const state = {
      v: 1,
      w: 100,
      h: 100,
      e: [
        { t: 'd', x: 0.5, y: 0.5, r: 0.03, c: [1, 0, 0], i: 5.0 },
        { t: 'a', x: 0.7, y: 0.5, r: 0.1, s: 5.0 },
      ],
    };
    const encoded = toBase64Url(JSON.stringify(state));

    await page.goto('/#s=' + encoded);

    const canvas = page.locator('#the_canvas');
    await expect(canvas).toBeVisible({ timeout: 10000 });

    // Wait a bit for WASM init and log propagation
    await page.waitForTimeout(1000);

    // Expect our info log from the app
    const hasApplied = logs.some((l) => l.text.includes('Applied share state from URL'));
    expect(hasApplied).toBeTruthy();

    // Page URL should keep our s= parameter
    expect(page.url()).toContain('#s=');

    // Quick screenshot artifact
    await page.screenshot({ path: 'test-results/share-url-loaded.png' });
  });

  test('updates URL hash when adding pinned elements', async ({ page }) => {
    await page.goto('/');

    const canvas = page.locator('#the_canvas');
    await expect(canvas).toBeVisible({ timeout: 10000 });

    // Enable placement mode (📌) and ensure Dye tool (🎨) is selected
    // Click the pin toggle
    await page.getByText('📌').click();

    // Place a persistent dye source with a simple click near center
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    const x = box.x + box.width * 0.5;
    const y = box.y + box.height * 0.5;
    await page.mouse.click(x, y);

    // Allow app to serialize and update URL
    await page.waitForTimeout(500);

    // Read hash and decode our state
    const url = new URL(page.url());
    const hash = url.hash.startsWith('#') ? url.hash.substring(1) : url.hash;
    expect(hash).toContain('s=');
    const sVal = hash.split('&').find((p) => p.startsWith('s='));
    expect(sVal).toBeTruthy();
    const b64 = sVal.substring(2);

    // Convert base64url to JSON
    const pad = (str) => str + '==='.slice((str.length + 3) % 4);
    const b64std = pad(b64.replace(/-/g, '+').replace(/_/g, '/'));
    const json = Buffer.from(b64std, 'base64').toString('utf-8');
    const parsed = JSON.parse(json);

    expect(parsed).toHaveProperty('e');
    expect(Array.isArray(parsed.e)).toBeTruthy();
    expect(parsed.e.length).toBeGreaterThan(0);

    await page.screenshot({ path: 'test-results/share-url-added.png' });
  });
});

