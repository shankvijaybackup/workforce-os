# Workforce OS

Workforce OS is an enterprise-grade operational intelligence platform designed to map organizational friction without compromising employee privacy. It utilizes edge-level cryptography and deterministic heuristics to understand "Deep Work" versus "Administrative Fragmentation."

## Enterprise Use Cases

### 1. Algorithmic Burnout Detection
Identify systemic workflow fragmentation before your top engineers burn out. By tracking the mathematical entropy of keystrokes and context switches, Workforce OS can surface teams experiencing 400+ context switches a day without generating proportional deep work. 
* **Key Metric:** Total Deep Work Hours vs Context Switches.

### 2. Shadow IT & Tool Consolidation
Discover exactly which SaaS applications drive collaboration and which are abandoned. Reclaim unused software licenses based on actual usage telemetry rather than self-reported surveys. 
* **Key Metric:** Secure, localized Application Hash Dictionary aggregations.

### 3. Zero-Surveillance Productivity Metrics
We don't record screens, we don't log keystrokes, and we don't read emails. All context mapping is done via mathematical hashing and AES-256-GCM encryption at the absolute edge. Telemetry is securely decrypted in a volatile Go enclave and aggregated in ClickHouse.

## Architecture
1. **Sensors:** Native zero-knowledge agents for macOS (Rust/Darwin) and Windows (Rust/Win32).
2. **Buffer:** AWS Kinesis for scalable, high-throughput ingestion.
3. **Enclave:** Stateless Go worker pool executing volatile decryption.
4. **Data Warehouse:** ClickHouse Materialized Views for temporal aggregations.
5. **Dashboard:** Next.js App Router enforcing RBAC at the Vercel Edge.

## Deployment
Agents are distributed silently using MDM payloads (`.mobileconfig` for Jamf/Kandji) and WiX MSIs for Microsoft Intune. 

---
*Operational Intelligence. Zero Surveillance.*
