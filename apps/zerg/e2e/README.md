# End-to-End Tests with Playwright

This directory contains comprehensive Playwright tests for the Zerg fullstack application.

## Test Structure

- `api.spec.ts` - Tests for REST API endpoints and LangGraph agent API
- `regular-chat.spec.ts` - Tests for the regular HTTP chat interface
- `streaming-chat.spec.ts` - Tests for the streaming chat interface
- `navigation.spec.ts` - Tests for UI navigation and layout
- `simple.spec.ts` - Basic smoke tests for core functionality

## Running Tests

### Prerequisites

Make sure all services are running:
```bash
just dev
```

### Test Commands

```bash
# Run all tests
just test

# Run tests with UI mode
just test-ui

# Run tests in headed mode (visible browser)
just test-headed

# Run simple smoke tests only
just test-simple

# Run tests in debug mode
just test-debug

# Run specific test file
bunx playwright test e2e/api.spec.ts

# Run tests for specific browser
bunx playwright test --project=chromium
bunx playwright test --project=firefox
bunx playwright test --project=webkit
```

## Test Configuration

The tests are configured in `playwright.config.ts` to:
- Start all required services automatically (API, ts-agent, UI)
- Run on multiple browsers (Chromium, Firefox, WebKit)
- Take screenshots on failure
- Generate HTML reports
- Use proper timeouts for service startup

## Services Tested

1. **Rust API** (Port 9080)
   - `/tasks` endpoint
   - `/api/chat` endpoint
   - `/api/chat/stream` endpoint
   - `/metrics` endpoint

2. **LangGraph Agent** (Port 3001)
   - `/chat` endpoint

3. **SolidJS UI** (Port 3000)
   - Regular chat interface
   - Streaming chat interface
   - Navigation and layout
   - Form interactions

## Test Features

- **API Testing**: Direct HTTP requests to test backend functionality
- **UI Testing**: Browser automation to test user interactions
- **Cross-browser Testing**: Ensures compatibility across different browsers
- **Visual Testing**: Screenshots on failures for debugging
- **Parallel Execution**: Tests run in parallel for faster execution
- **Retry Logic**: Automatic retries in CI environments

## Debugging

- Use `just test-debug` to run tests with Playwright Inspector
- Check `test-results/` directory for screenshots and traces on failures
- View HTML reports at `http://localhost:9323` after test runs
- Use `--headed` flag to see browser actions in real-time

## CI/CD Integration

The tests are configured to work in CI environments with:
- Increased retry counts
- Single worker execution
- Proper service startup detection
- Comprehensive reporting