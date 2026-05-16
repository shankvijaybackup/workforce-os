import React from 'react';

export default function DocsHome() {
  return (
    <div className="docs-container">
      <header className="docs-header">
        <h1>Technical Documentation Hub</h1>
        <p>Enterprise deployment guidelines and schema definitions for Workforce OS.</p>
      </header>

      <div className="docs-content">
        <section className="docs-section">
          <h2>1. Agent Architecture</h2>
          <p>
            The telemetry agents bypass Electron and heavy frameworks to interface directly with OS-level hooks:
          </p>
          <ul>
            <li><strong>macOS (Darwin):</strong> Native Rust binary using <code>Accessibility</code> and <code>CoreFoundation</code>.</li>
            <li><strong>Windows (Win32):</strong> Native Rust binary utilizing <code>User32</code> Hooks and executing as <code>NT AUTHORITY\LocalService</code>.</li>
          </ul>
          <div className="alert-item">
            <strong>Performance Constraint:</strong> Agents are mathematically restricted to a sub-20MB memory footprint and sub-1% CPU utilization.
          </div>
        </section>

        <section className="docs-section">
          <h2>2. Ingress Schema V1.1</h2>
          <p>The interactive JSON payload definition utilized by the Go Intelligence Enclave.</p>
          <pre className="code-block">
{`{
  "event_id": "uuid-v4",
  "tenant_id": "uuid-v4",
  "user_id": "uuid-v4",
  "timestamp": "ISO-8601",
  "payload": {
    "app_bundle_hash": "SHA-256",
    "window_title_hash": "SHA-256",
    "duration_seconds": 120,
    "keystroke_entropy": 0.85
  },
  "auth_tag": "AES-256-GCM-Tag"
}`}
          </pre>
          <p>
            <em>Note: All payloads are AES-256-GCM encrypted at the edge. The Kinesis buffer never stores plaintext strings.</em>
          </p>
        </section>

        <section className="docs-section">
          <h2>3. Deployment Guides</h2>
          <h3>Windows Fleet (.msi)</h3>
          <p>Deploy silently via Microsoft Intune or SCCM using the packaged WiX installer:</p>
          <pre className="code-block">msiexec /i workforce-agent.msi /qn TENANT_ID="YOUR-TENANT-ID"</pre>
          
          <h3>macOS Fleet (.mobileconfig)</h3>
          <p>
            Upload the <code>workforce-os-mdm.mobileconfig</code> to Jamf or Kandji to silently grant PPPC 
            Accessibility permissions and register the <code>launchd</code> background task before deploying the signed binary.
          </p>
        </section>
      </div>
    </div>
  );
}
