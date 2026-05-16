import React from 'react';

export default function MarketingHome() {
  return (
    <div className="marketing-container">
      {/* Hero Section */}
      <section className="hero-section">
        <h1 className="hero-headline">Operational Intelligence.<br/>Zero Surveillance.</h1>
        <p className="hero-copy">
          Workforce OS relies on cryptographic masking and edge-level entropy modeling. 
          We don't record screens, we don't log keystrokes, and we don't read emails. 
          We map the friction in your organization using deterministic physics.
        </p>
      </section>

      {/* Solutions Grid */}
      <section className="solutions-section">
        <div className="solution-card">
          <div className="solution-indicator solid"></div>
          <h2>Algorithmic Burnout Detection</h2>
          <p>
            <strong>Identify systemic workflow fragmentation before your top engineers burn out.</strong><br/>
            Surface teams experiencing 400+ context switches a day without generating proportional deep work. 
            Powered by our deterministic DEEP_WORK vs ADMINISTRATIVE temporal heuristic engine.
          </p>
        </div>

        <div className="solution-card">
          <div className="solution-indicator striped"></div>
          <h2>Shadow IT &amp; Tool Consolidation</h2>
          <p>
            <strong>Discover exactly which SaaS applications drive collaboration and which are abandoned.</strong><br/>
            Reclaim unused licenses without relying on self-reported surveys. 
            Driven by our secure, localized Application Hash Dictionary.
          </p>
        </div>
      </section>
    </div>
  );
}
