import { test, expect } from '@playwright/test';

/**
 * E2E tests for itsliquid WASM simulation
 * Tests WASM module loading, console logs, and user interactions
 */

test.describe('WASM Module Loading', () => {
  test('should load the page and display loading indicator', async ({ page }) => {
    await page.goto('/');

    // Check that loading indicator is initially visible
    const loading = page.locator('.loading');
    await expect(loading).toBeVisible({ timeout: 5000 });
  });

  test('should successfully initialize WASM module', async ({ page }) => {
    // Set up console log capture BEFORE navigating
    const consoleLogs = [];
    page.on('console', msg => {
      consoleLogs.push({
        type: msg.type(),
        text: msg.text(),
        timestamp: new Date().toISOString()
      });
    });

    await page.goto('/');

    // Wait for loading to complete (max 10 seconds)
    const loading = page.locator('.loading');
    await expect(loading).toHaveCSS('display', 'none', { timeout: 10000 });

    // Check that canvas is now visible
    const canvas = page.locator('#the_canvas');
    await expect(canvas).toBeVisible();

    // Verify console logs show successful initialization
    const hasStartLog = consoleLogs.some(log =>
      log.text.includes('Starting itsliquid WASM')
    );
    expect(hasStartLog).toBeTruthy();

    const hasSuccessLog = consoleLogs.some(log =>
      log.text.includes('eframe started successfully')
    );
    expect(hasSuccessLog).toBeTruthy();

    console.log('Captured console logs:', JSON.stringify(consoleLogs, null, 2));
  });

  test('should not have any console errors during initialization', async ({ page }) => {
    const consoleErrors = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    await page.goto('/');

    // Wait for initialization
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Check for errors
    expect(consoleErrors).toHaveLength(0);
    if (consoleErrors.length > 0) {
      console.error('Console errors found:', consoleErrors);
    }
  });
});

test.describe('Canvas and Rendering', () => {
  test('canvas should have correct dimensions', async ({ page }) => {
    await page.goto('/');

    // Wait for canvas to be visible
    const canvas = page.locator('#the_canvas');
    await expect(canvas).toBeVisible({ timeout: 10000 });

    // Get canvas dimensions
    const boundingBox = await canvas.boundingBox();
    expect(boundingBox).not.toBeNull();
    expect(boundingBox.width).toBeGreaterThan(0);
    expect(boundingBox.height).toBeGreaterThan(0);

    console.log('Canvas dimensions:', boundingBox);
  });

  test('canvas should render content', async ({ page }) => {
    await page.goto('/');

    const canvas = page.locator('#the_canvas');
    await expect(canvas).toBeVisible({ timeout: 10000 });

    // Wait a bit for rendering to happen
    await page.waitForTimeout(1000);

    // Take a screenshot to verify visual output
    // Note: We screenshot the page instead of the canvas directly because
    // the canvas is continuously animating, which prevents Playwright from
    // waiting for a "stable" state. The page screenshot still captures the canvas.
    const screenshot = await page.screenshot({ path: 'test-results/canvas-initial-render.png' });

    // Verify screenshot has content
    expect(screenshot.length).toBeGreaterThan(1000);
  });
});

test.describe('User Interactions', () => {
  test('should handle left-click drag (fluid force)', async ({ page }) => {
    const consoleLogs = [];
    page.on('console', msg => {
      consoleLogs.push({
        type: msg.type(),
        text: msg.text()
      });
    });

    await page.goto('/');

    const canvas = page.locator('#the_canvas');
    await expect(canvas).toBeVisible({ timeout: 10000 });

    // Get canvas position and size
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    // Calculate positions relative to canvas (center and offset)
    const startX = box.x + box.width / 2;
    const startY = box.y + box.height / 2;
    const endX = startX + 50;
    const endY = startY + 50;

    console.log(`Simulating drag from (${startX}, ${startY}) to (${endX}, ${endY})`);

    // Perform left-click drag
    await page.mouse.move(startX, startY);
    await page.mouse.down();
    await page.mouse.move(endX, endY, { steps: 10 });
    await page.mouse.up();

    // Wait for simulation to process the interaction
    await page.waitForTimeout(500);

    // Take screenshot after interaction
    await page.screenshot({ path: 'test-results/after-drag-interaction.png' });

    console.log('Console logs after drag:', consoleLogs.slice(-10));
  });

  test('should handle right-click (add dye)', async ({ page }) => {
    await page.goto('/');

    const canvas = page.locator('#the_canvas');
    await expect(canvas).toBeVisible({ timeout: 10000 });

    // Get canvas position
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    // Calculate click position (center of canvas)
    const clickX = box.x + box.width / 2;
    const clickY = box.y + box.height / 2;

    console.log(`Simulating right-click at (${clickX}, ${clickY})`);

    // Take screenshot before
    await page.screenshot({ path: 'test-results/before-dye.png' });

    // Perform right-click to add dye
    await page.mouse.click(clickX, clickY, { button: 'right' });

    // Wait for dye to be added and rendered
    await page.waitForTimeout(500);

    // Take screenshot after
    await page.screenshot({ path: 'test-results/after-dye.png' });
  });

  test('should handle multiple interactions in sequence', async ({ page }) => {
    await page.goto('/');

    const canvas = page.locator('#the_canvas');
    await expect(canvas).toBeVisible({ timeout: 10000 });

    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    // Sequence of interactions:
    // 1. Add dye at position 1
    const pos1X = box.x + box.width * 0.3;
    const pos1Y = box.y + box.height * 0.3;
    await page.mouse.click(pos1X, pos1Y, { button: 'right' });
    await page.waitForTimeout(200);

    // 2. Add dye at position 2
    const pos2X = box.x + box.width * 0.7;
    const pos2Y = box.y + box.height * 0.3;
    await page.mouse.click(pos2X, pos2Y, { button: 'right' });
    await page.waitForTimeout(200);

    // 3. Drag to create fluid motion
    await page.mouse.move(pos1X, pos1Y);
    await page.mouse.down();
    await page.mouse.move(pos2X, pos2Y, { steps: 15 });
    await page.mouse.up();

    // Wait for simulation to process
    await page.waitForTimeout(1000);

    // Take final screenshot
    await page.screenshot({ path: 'test-results/after-sequence.png' });
  });
});

test.describe('Console Log Monitoring', () => {
  test('should capture all console output types', async ({ page }) => {
    const allConsoleLogs = {
      log: [],
      info: [],
      warn: [],
      error: [],
      debug: []
    };

    page.on('console', msg => {
      const type = msg.type();
      if (allConsoleLogs[type]) {
        allConsoleLogs[type].push({
          text: msg.text(),
          timestamp: new Date().toISOString()
        });
      }
    });

    await page.goto('/');
    await page.waitForTimeout(3000);

    // Log all captured messages
    console.log('\n=== Captured Console Logs ===');
    console.log('INFO:', allConsoleLogs.info);
    console.log('LOG:', allConsoleLogs.log);
    console.log('WARN:', allConsoleLogs.warn);
    console.log('ERROR:', allConsoleLogs.error);
    console.log('DEBUG:', allConsoleLogs.debug);

    // Verify we got expected initialization logs
    const allInfo = allConsoleLogs.info.map(l => l.text).join(' ');
    expect(allInfo).toContain('Starting itsliquid WASM');
  });

  test('should detect simulation errors in console', async ({ page }) => {
    const errors = [];
    const warnings = [];

    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push({
          text: msg.text(),
          location: msg.location(),
          timestamp: new Date().toISOString()
        });
      } else if (msg.type() === 'warning') {
        warnings.push({
          text: msg.text(),
          timestamp: new Date().toISOString()
        });
      }
    });

    // Also capture page errors
    page.on('pageerror', error => {
      errors.push({
        text: error.message,
        stack: error.stack,
        timestamp: new Date().toISOString()
      });
    });

    await page.goto('/');

    // Wait and interact
    const canvas = page.locator('#the_canvas');
    await expect(canvas).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(2000);

    // Perform some interactions
    const box = await canvas.boundingBox();
    if (box) {
      await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
      await page.waitForTimeout(1000);
    }

    // Report any errors or warnings
    if (errors.length > 0) {
      console.error('ERRORS DETECTED:', JSON.stringify(errors, null, 2));
    }
    if (warnings.length > 0) {
      console.warn('WARNINGS DETECTED:', JSON.stringify(warnings, null, 2));
    }

    // Test passes if no errors (warnings are okay)
    expect(errors).toHaveLength(0);
  });
});

test.describe('Performance and Stability', () => {
  test('should run simulation for 10 seconds without crashes', async ({ page }) => {
    const consoleLogs = [];
    const errors = [];

    page.on('console', msg => {
      consoleLogs.push({
        type: msg.type(),
        text: msg.text(),
        timestamp: Date.now()
      });
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    await page.goto('/');

    const canvas = page.locator('#the_canvas');
    await expect(canvas).toBeVisible({ timeout: 10000 });

    // Run for 10 seconds
    console.log('Starting 10-second stability test...');
    await page.waitForTimeout(10000);

    // Canvas should still be visible
    await expect(canvas).toBeVisible();

    // No errors should have occurred
    expect(errors).toHaveLength(0);

    console.log(`Simulation ran for 10 seconds. Captured ${consoleLogs.length} console messages.`);
  });

  test('should handle rapid user interactions', async ({ page }) => {
    await page.goto('/');

    const canvas = page.locator('#the_canvas');
    await expect(canvas).toBeVisible({ timeout: 10000 });

    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    // Perform rapid clicks
    console.log('Performing 20 rapid clicks...');
    for (let i = 0; i < 20; i++) {
      const x = box.x + Math.random() * box.width;
      const y = box.y + Math.random() * box.height;
      await page.mouse.click(x, y, { button: 'right' });
      await page.waitForTimeout(50); // Small delay between clicks
    }

    // Wait for simulation to stabilize
    await page.waitForTimeout(1000);

    // Canvas should still be responsive
    await expect(canvas).toBeVisible();

    // Take screenshot
    await page.screenshot({ path: 'test-results/after-rapid-clicks.png' });
  });
});
