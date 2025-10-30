# Browser Testing Guide for itsliquid WASM

This document provides detailed information about the automated browser testing system for the itsliquid fluid simulation.

## Overview

The testing system uses **Playwright** to run automated end-to-end tests in real browsers (Firefox, Chromium). Tests verify:

- ✅ WASM module loads correctly
- ✅ Rust code initializes without errors
- ✅ Console logs are captured (from Rust via `console_log` crate)
- ✅ User interactions work (clicks, drags)
- ✅ Visual rendering produces output
- ✅ System remains stable under load

## Quick Start

```bash
# From project root
./test-web.sh
```

This single command:
1. Builds the WASM module
2. Installs npm dependencies (if needed)
3. Installs browser binaries (if needed)
4. Runs all tests in headless Firefox
5. Generates HTML report with screenshots

## Command Line Options

```bash
./test-web.sh                # Default: headless Firefox
./test-web.sh --headed       # Visible browser window
./test-web.sh --chromium     # Use Chromium instead of Firefox
./test-web.sh --debug        # Interactive debugger (step through tests)
./test-web.sh --ui           # Playwright UI mode (explore tests)
```

## Test Structure

### Test File Location
`web/tests/wasm-simulation.spec.js`

### Test Suites

#### 1. WASM Module Loading
Tests the initialization sequence:
- Loading indicator appears
- WASM module loads successfully
- Canvas becomes visible
- Console shows "Starting itsliquid WASM..."
- Console shows "eframe started successfully!"
- No JavaScript errors occur

#### 2. Canvas and Rendering
Verifies visual output:
- Canvas has correct dimensions
- Canvas is visible after initialization
- Rendering produces non-blank content
- Screenshots are captured for visual inspection

#### 3. User Interactions
Simulates real user behavior:

**Left-click drag** (Fluid Force):
```javascript
// Drags from center to offset position
await page.mouse.move(startX, startY);
await page.mouse.down();
await page.mouse.move(endX, endY, { steps: 10 });
await page.mouse.up();
```

**Right-click** (Add Dye):
```javascript
// Clicks at specific canvas position
await page.mouse.click(clickX, clickY, { button: 'right' });
```

**Sequences**:
- Multiple dye additions
- Drag between dye spots
- Rapid-fire interactions

#### 4. Console Monitoring
Captures all console output from both JavaScript and Rust:

```javascript
page.on('console', msg => {
  consoleLogs.push({
    type: msg.type(),      // 'log', 'info', 'warn', 'error', 'debug'
    text: msg.text(),      // Message content
    timestamp: new Date()  // When it occurred
  });
});
```

Rust logs appear via the `console_log` crate:
```rust
log::info!("Starting itsliquid WASM...");
log::error!("Failed to start eframe: {:?}", e);
```

#### 5. Performance & Stability
Stress tests:
- 10-second continuous run
- 20 rapid clicks in random positions
- Memory stability checks
- Error detection during extended use

## Console Log Format

The Rust code logs to the browser console via `console_log::init_with_level()`:

| Rust Log Level | Browser Console | Test Capture |
|----------------|-----------------|--------------|
| `log::info!()` | `console.info()` | `type: 'info'` |
| `log::warn!()` | `console.warn()` | `type: 'warning'` |
| `log::error!()` | `console.error()` | `type: 'error'` |
| `log::debug!()` | `console.log()` | `type: 'log'` |

Example from `src/lib.rs:65-67`:
```rust
log::info!("Starting itsliquid WASM...");
log::info!("Creating WebRunner...");
log::info!("eframe started successfully!");
```

These appear in test output as:
```json
{
  "type": "info",
  "text": "Starting itsliquid WASM...",
  "timestamp": "2025-10-30T12:34:56.789Z"
}
```

## Test Results

### Screenshots
All screenshots are saved to `web/test-results/`:

| File | Captures |
|------|----------|
| `canvas-initial-render.png` | Canvas after WASM loads |
| `after-drag-interaction.png` | After left-click drag (fluid force) |
| `before-dye.png` | Before adding dye |
| `after-dye.png` | After right-click (dye added) |
| `after-sequence.png` | After multi-step interaction |
| `after-rapid-clicks.png` | After 20 rapid clicks |

### HTML Report
View detailed results with videos and traces:

```bash
cd web
npx playwright show-report
```

The report includes:
- ✅ Pass/fail status for each test
- ⏱️ Execution times
- 📹 Videos of test runs (on failure)
- 📊 Traces for debugging (on retry)
- 📸 Screenshots at each step

### Console Output
During test execution, you'll see:

```
🧪 itsliquid WASM Browser Testing
==================================

📦 Step 1: Building WASM...
✅ WASM build complete

📥 Step 2: Installing npm dependencies...
✅ Dependencies installed

🌐 Step 3: Installing browser binaries...
✅ Browsers installed

✅ Step 4: Test results directory ready

🚀 Step 5: Running E2E tests...

Running 13 tests using 1 worker

  ✓  1 [firefox] › wasm-simulation.spec.js:6:3 › WASM Module Loading › should load...
  ✓  2 [firefox] › wasm-simulation.spec.js:13:3 › WASM Module Loading › should init...
  ✓  3 [firefox] › wasm-simulation.spec.js:45:3 › WASM Module Loading › no errors...
  ...

✅ All tests passed!
```

## Manual npm Commands

```bash
cd web

# Install dependencies
npm install
npm run install-browsers

# Run tests
npm test                    # Default: headless Firefox
npm run test:headed         # Visible browser
npm run test:firefox        # Firefox specifically
npm run test:chromium       # Chromium specifically
npm run test:debug          # Step-by-step debugger
npm run test:ui             # Interactive UI mode

# View reports
npx playwright show-report
```

## Configuration

### playwright.config.js
Key settings:

```javascript
{
  testDir: './tests',
  timeout: 30000,           // 30s per test
  use: {
    baseURL: 'http://localhost:8000',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  webServer: {
    command: 'python3 -m http.server 8000',
    url: 'http://localhost:8000',
    timeout: 120000,        // 2 min startup timeout
  }
}
```

The `webServer` automatically starts a local server before tests and stops it after.

## Debugging Tests

### Method 1: Headed Mode
See the browser in action:
```bash
./test-web.sh --headed
```

### Method 2: Debug Mode
Step through tests with Playwright Inspector:
```bash
./test-web.sh --debug
```

Features:
- Pause before each action
- Step forward/backward
- Inspect page state
- View console logs
- Edit locators live

### Method 3: UI Mode
Explore tests interactively:
```bash
./test-web.sh --ui
```

Features:
- Visual test tree
- Time-travel debugging
- Live DOM snapshots
- Network activity
- Console logs

### Method 4: Console Logs
Add custom logging to tests:

```javascript
test('my test', async ({ page }) => {
  const logs = [];
  page.on('console', msg => {
    logs.push(msg.text());
    console.log('BROWSER:', msg.text());
  });

  // ... test actions ...

  console.log('All logs:', logs);
});
```

## Troubleshooting

### "Browser not found"
Run:
```bash
cd web && npm run install-browsers
```

### "Port 8000 already in use"
Kill existing server:
```bash
lsof -ti:8000 | xargs kill -9
```

Or change port in `playwright.config.js`:
```javascript
webServer: {
  command: 'python3 -m http.server 8001',
  url: 'http://localhost:8001',
}
```

### WASM fails to load
Check build output:
```bash
wasm-pack build --target web --out-dir web/pkg --release
ls -lh web/pkg/
```

Should see:
- `itsliquid_bg.wasm` (the binary)
- `itsliquid.js` (bindings)
- `itsliquid.d.ts` (TypeScript types)
- `package.json`

### Console logs not appearing
Verify Rust logging is initialized in `src/lib.rs:62-63`:
```rust
console_error_panic_hook::set_once();
console_log::init_with_level(log::Level::Debug).ok();
```

### Tests timeout
Increase timeout in test file:
```javascript
test('my test', async ({ page }) => {
  test.setTimeout(60000); // 60 seconds
  // ...
});
```

## CI/CD Integration

For continuous integration:

```yaml
# .github/workflows/test.yml
name: WASM Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install wasm-pack
        run: cargo install wasm-pack

      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: 18

      - name: Run tests
        run: ./test-web.sh

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: web/test-results/
```

## Writing New Tests

Add tests to `web/tests/wasm-simulation.spec.js`:

```javascript
test.describe('My New Feature', () => {
  test('should do something', async ({ page }) => {
    // Navigate to the app
    await page.goto('/');

    // Wait for WASM to load
    const canvas = page.locator('#the_canvas');
    await expect(canvas).toBeVisible({ timeout: 10000 });

    // Capture console logs
    const logs = [];
    page.on('console', msg => {
      logs.push(msg.text());
    });

    // Perform actions
    const box = await canvas.boundingBox();
    await page.mouse.click(box.x + 50, box.y + 50);

    // Assert results
    await page.waitForTimeout(500);
    expect(logs).toContain('Expected log message');

    // Take screenshot
    await page.screenshot({ path: 'test-results/my-feature.png' });
  });
});
```

## Best Practices

1. **Always wait for canvas visibility** - The WASM module takes time to initialize
2. **Use boundingBox for clicks** - Calculate positions relative to canvas
3. **Capture console logs early** - Set up listeners before `page.goto()`
4. **Add small delays after actions** - Let simulation process interactions (200-500ms)
5. **Take screenshots for visual tests** - Helps debug rendering issues
6. **Log captured console output** - Use `console.log()` in tests to debug
7. **Use specific timeouts** - Don't rely on global timeout for critical waits

## Resources

- [Playwright Documentation](https://playwright.dev/)
- [Playwright Test API](https://playwright.dev/docs/api/class-test)
- [Browser Automation Patterns](https://playwright.dev/docs/best-practices)
- [Debugging Guide](https://playwright.dev/docs/debug)
