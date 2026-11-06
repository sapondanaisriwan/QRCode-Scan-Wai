# AGENTS.md

## 1. Project Overview

**Project Name:** ScreenQR

ScreenQR is a lightweight Windows desktop application that allows users to scan QR codes directly from their computer screens without using a camera or mobile device.

The application must operate locally, prioritize user privacy, and provide a fast, intuitive experience similar to Windows Snipping Tool.

**Target Platforms:**
- Windows 10 (64-bit)
- Windows 11 (64-bit)

**Distribution:** Microsoft Store

### Core Features

1. Capture QR codes from the user's screen.
2. Select a screen region using a mouse-driven overlay.
3. Decode QR codes locally.
4. Trigger scanning using a global keyboard shortcut.
5. Display decoded results.
6. Copy decoded content to the clipboard.
7. Open scanned URLs only after explicit user action.
8. Support system tray integration.
9. Optionally store scan history locally.
10. Support multiple monitors and DPI scaling.

## 2. Technology Stack

Use the following technologies unless a change is technically justified and approved.

| Layer | Technology |
|---|---|
| Desktop Framework | Tauri v2 |
| Frontend | React + TypeScript |
| Build Tool | Vite |
| Styling | Tailwind CSS v4 |
| Native Logic | Rust |
| Screen Capture | Windows-compatible native APIs or maintained Rust crates |
| QR Decoder | Rust-based QR decoding library |
| Local Storage | JSON or SQLite when necessary |
| Package Manager | npm |
| Distribution | Microsoft Store |

### Restrictions

- Do not introduce a remote backend server.
- Do not introduce authentication or user accounts.
- Do not introduce cloud services for QR decoding.
- Do not use Electron.
- Do not add dependencies without a clear technical reason.
- Prefer maintained, stable libraries compatible with Tauri v2.

## 3. Architecture Principles

Follow a modular architecture with clear separation of responsibilities.

### Frontend Responsibilities

The React frontend handles:

- Application UI and navigation.
- Scanner controls.
- Scan results.
- User interaction.
- Loading and error states.
- Application settings.
- Displaying scan history.

The frontend must not directly access native Windows APIs.

### Rust Responsibilities

The Rust layer handles:

- Screen capture.
- Image processing.
- QR code detection and decoding.
- Native operating system integration.
- Global keyboard shortcuts.
- System tray behavior.
- Local file operations where required.

### Communication

Use Tauri IPC commands to communicate between React and Rust.

- Keep IPC interfaces small and clearly typed.
- Validate all inputs received by Rust commands.
- Return structured results and meaningful errors.
- Avoid transferring unnecessarily large images across IPC.
- Do not expose unrestricted native capabilities to the frontend.

## 4. Suggested Project Structure

```text
screenqr/
├── AGENTS.md
├── README.md
├── docs/
│   ├── REQUIREMENTS.md
│   ├── ARCHITECTURE.md
│   ├── DEVELOPMENT_PLAN.md
│   └── TEST_PLAN.md
├── src/
│   ├── components/
│   │   ├── ui/
│   │   └── layout/
│   ├── features/
│   │   ├── scanner/
│   │   ├── results/
│   │   ├── history/
│   │   └── settings/
│   ├── hooks/
│   ├── lib/
│   ├── types/
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/
│   ├── src/
│   │   ├── commands/
│   │   ├── capture/
│   │   ├── decoder/
│   │   ├── shortcuts/
│   │   ├── storage/
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── capabilities/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── vite.config.ts
```

This structure is a guideline, not a requirement to create empty files or directories.

Follow the existing repository structure when it already provides an appropriate organization.

## 5. Coding Standards

### TypeScript and React

- Use TypeScript strict mode.
- Prefer functional React components.
- Use hooks for component behavior.
- Use explicit types for IPC responses.
- Avoid `any` unless absolutely necessary.
- Keep components small and focused.
- Separate business logic from presentation.
- Handle loading, success, empty, and error states.
- Prefer existing project conventions over introducing new patterns.
- Avoid unnecessary state management libraries.

### Rust

- Follow idiomatic Rust conventions.
- Use `Result<T, E>` for fallible operations.
- Avoid `unwrap()` and `expect()` in production paths where failure is possible.
- Use meaningful error types.
- Separate screen capture from QR decoding.
- Release native resources correctly.
- Avoid unnecessary cloning and memory allocation.
- Keep platform-specific code isolated.
- Use `cargo fmt` and `cargo clippy`.

### Naming Conventions

**TypeScript:**
- Components: PascalCase
- Functions and variables: camelCase
- Constants: UPPER_SNAKE_CASE when appropriate
- Types and interfaces: PascalCase

**Rust:**
- Functions and modules: snake_case
- Structs and enums: PascalCase
- Constants: UPPER_SNAKE_CASE

## 6. Screen Capture Requirements

Screen capture is a critical feature.

### Required Behavior

1. The user initiates a scan.
2. Capture the appropriate screen content.
3. Display a transparent region-selection overlay.
4. Allow the user to select a rectangular area.
5. Crop the captured image using the selected coordinates.
6. Decode QR codes within the cropped region.
7. Display the result.
8. Release temporary image data.

### Important Constraints

- Do not allow the selection overlay to obscure the captured content.
- Handle Windows display scaling correctly.
- Account for multiple monitors.
- Handle negative monitor coordinates.
- Support cancellation using the Escape key.
- Avoid permanently saving screenshots.
- Handle capture failures gracefully.
- Do not attempt to bypass protected content or Windows security restrictions.

Prefer capturing screen content before displaying the selection overlay when technically appropriate.

## 7. QR Decoding Requirements

The QR decoder must:

- Decode standard QR codes.
- Support URL and plain-text content.
- Preserve UTF-8 text, including Thai characters.
- Handle invalid and unreadable images gracefully.
- Avoid network requests.
- Return the original decoded content without modifying it.

### Result Structure

Prefer a typed result similar to:

```typescript
type ScanResult = {
  content: string;
  type: "url" | "text" | "other";
};
```

Do not classify arbitrary text as a safe URL without validation.

## 8. Security and Privacy

Privacy and security are core project requirements.

### Screen Capture Privacy

- Process captured images locally.
- Never upload screenshots to external services.
- Do not save screenshots automatically.
- Do not capture screens continuously in the background.
- Start screen capture only after explicit user action.
- Release temporary image buffers when no longer needed.

### QR Code Safety

Treat all decoded QR content as untrusted input.

- Never execute decoded content as commands.
- Never automatically open scanned URLs.
- Never interpret QR content as HTML.
- Do not use unsafe HTML rendering.
- Validate URL schemes before opening external links.
- Prefer allowing only `http` and `https` URLs for browser-opening actions.
- Display the destination URL before opening it.
- Never automatically connect to Wi-Fi networks or execute embedded instructions.

### Local Storage

- Store only necessary data.
- Allow users to disable scan history.
- Allow users to clear scan history.
- Avoid logging sensitive QR content.
- Do not introduce analytics or telemetry without explicit approval.

## 9. UI/UX Guidelines

The application should feel like a native Windows utility.

### Design Principles

- Minimal and clean interface.
- Fast access to scanning.
- Clear visual hierarchy.
- Consistent spacing and typography.
- Accessible keyboard navigation.
- Light and dark theme support.
- Responsive layout within desktop window constraints.

### User Experience

The primary workflow should be:

**Launch → Scan → Select Region → Decode → Copy/Open**

Avoid unnecessary confirmation dialogs except for potentially sensitive operations.

### Error Handling

Provide meaningful feedback for:

- No QR code detected.
- Invalid or unreadable QR code.
- Screen capture failure.
- Shortcut registration failure.
- Clipboard operation failure.
- Unsupported URL scheme.

Never expose raw Rust panic messages to end users.

## 10. Performance Requirements

Use these as initial performance targets, subject to testing on representative Windows hardware.

- QR decoding should typically complete within 1 second after selection.
- Application startup should target less than 3 seconds.
- Idle memory usage should target less than 150 MB.
- Avoid blocking the React UI thread.
- Avoid unnecessary background processing.
- Avoid repeated full-screen captures.
- Release image buffers after processing.

Do not claim that a performance target has been achieved without measurement.

## 11. Testing Requirements

### Rust Tests

Write tests for:

- QR decoding.
- UTF-8 content.
- URL classification.
- Invalid image handling.
- Empty or unreadable images.
- Coordinate and crop calculations.

### Frontend Tests

Test:

- Scanner UI states.
- Scan result rendering.
- Copy button behavior.
- Error messages.
- URL opening controls.
- Settings behavior when implemented.

### Windows Integration Tests

Manually verify:

- Global shortcut while another application is focused.
- Region selection.
- Escape cancellation.
- Multiple monitors.
- Different DPI scaling configurations.
- Windows 10 and Windows 11 compatibility.
- Application startup and shutdown.
- System tray behavior when implemented.

Do not claim Windows-specific features work based only on successful compilation.

## 12. Development Workflow

Follow an incremental development process.

### Phase 1: Project Foundation

- Initialize and verify Tauri.
- Configure React, TypeScript, and Tailwind CSS.
- Establish project structure.
- Configure formatting and linting.

### Phase 2: QR Decoder

- Implement QR decoding in Rust.
- Expose a typed Tauri command.
- Add automated decoder tests.

### Phase 3: Screen Capture

- Implement Windows screen capture.
- Implement region selection.
- Integrate cropping and decoding.
- Handle cancellation and errors.

### Phase 4: Frontend Integration

- Implement the main window.
- Implement scan controls.
- Display scan results.
- Implement copy and open actions.

### Phase 5: Desktop Integration

- Add global shortcuts.
- Add system tray integration.
- Add settings and optional local history.
- Improve multi-monitor support.

### Phase 6: Testing and Release

- Run automated tests.
- Perform Windows integration testing.
- Review privacy and security.
- Build release packages.
- Prepare Microsoft Store submission.

## 13. Instructions for AI Coding Agents

Before making changes:

1. Read `AGENTS.md`.
2. Read `docs/REQUIREMENTS.md` if available.
3. Inspect relevant existing code.
4. Identify the current development phase.
5. Explain the proposed changes for non-trivial tasks.

During implementation:

- Implement only the requested scope.
- Prefer small, reviewable changes.
- Do not rewrite unrelated code.
- Do not introduce unnecessary abstractions.
- Do not install dependencies without justification.
- Follow the established architecture.
- Add or update tests when appropriate.
- Preserve existing functionality.
- Ask for clarification when requirements are ambiguous or conflicting.

After implementation:

1. Format changed code.
2. Run relevant linting and type checks.
3. Run relevant automated tests.
4. Run the build when appropriate.
5. Summarize changes.
6. Report tests performed and their results.
7. Explicitly disclose untested behavior and remaining issues.

### Preferred Validation Commands

Use applicable commands from the repository:

```bash
npm run build
cargo fmt --check
cargo clippy --all-targets
cargo test
```

Run Rust commands from `src-tauri` when appropriate.

Do not assume a command succeeded without checking its output.

## 14. Documentation Rules

Keep documentation consistent with implementation.

- Update `README.md` when setup or usage changes.
- Update architecture documentation when architectural decisions change.
- Update development plans when milestones are completed.
- Document important Windows-specific limitations.
- Document new dependencies and their purpose.
- Do not mark features as complete until verified.

## 15. Microsoft Store Requirements

The application is intended for Microsoft Store distribution.

- Use a stable application identifier.
- Maintain appropriate application metadata.
- Avoid unnecessary permissions.
- Provide a clear privacy policy.
- Prepare application icons and screenshots.
- Verify packaging compatibility.
- Test installation and uninstallation.
- Review current Microsoft Store certification requirements before submission.

Never publish or submit an application without explicit user approval.

## 16. Out of Scope

Unless explicitly requested, do not implement:

- Cloud backend services.
- Authentication or account management.
- Cloud synchronization.
- QR code generation.
- Camera-based QR scanning.
- Barcode scanning beyond QR codes.
- Payment or subscription systems.
- Analytics or tracking.
- Automatic execution of QR content.
- Continuous background screen monitoring.

## 17. Definition of Done

A feature is considered complete only when:

1. Its functional requirements are implemented.
2. Relevant error cases are handled.
3. Appropriate tests are added or updated.
4. Relevant checks pass.
5. Windows-specific behavior is verified where required.
6. Documentation is updated when necessary.
7. No known critical security or privacy issues remain.

## 18. Guiding Principle

**Keep ScreenQR small, fast, private, and reliable.**

Prefer simple, maintainable solutions over unnecessary complexity.

The primary goal is to let users scan a QR code from their Windows screen quickly, safely, and without requiring a phone, camera, account, or internet connection.