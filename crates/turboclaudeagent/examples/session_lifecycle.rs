//! Session Lifecycle Example
//!
//! Demonstrates:
//! 1. Lifecycle event tracking
//! 2. RAII-style automatic cleanup with SessionGuard
//! 3. Resource management best practices
//! 4. Event-driven observability
//!
//! Run with: cargo run --example session_lifecycle

use turboclaudeagent::ClaudeAgentClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔄 Session Lifecycle Management Example\n");

    // ============================================================
    // SECTION 1: LIFECYCLE EVENTS
    // ============================================================
    println!("📋 Section 1: Understanding Session Lifecycle Events");
    println!("═════════════════════════════════════════════════════\n");

    demonstrate_lifecycle_events();

    // ============================================================
    // SECTION 2: RAII AUTO-CLEANUP (if running with real API)
    // ============================================================
    println!("\n🔒 Section 2: RAII Pattern for Auto-Cleanup");
    println!("════════════════════════════════════════════\n");

    if std::env::var("CLAUDE_API_KEY").is_ok() {
        demonstrate_raii_cleanup().await?;
    } else {
        println!("⚠️  CLAUDE_API_KEY not set, skipping live demo");
        println!("\nRMAI Pattern (Rust's Resource Acquisition Is Initialization):");
        println!("  • SessionGuard ensures cleanup happens automatically");
        println!("  • No manual .close() calls needed");
        println!("  • Cleanup happens even if errors occur");
        println!("  • Rust compiler guarantees cleanup on scope exit\n");
    }

    // ============================================================
    // SECTION 3: EVENT TYPES & MEANINGS
    // ============================================================
    println!("📊 Section 3: Event Type Reference");
    println!("═══════════════════════════════════\n");

    describe_event_types();

    // ============================================================
    // SECTION 4: OBSERVABILITY PATTERNS
    // ============================================================
    println!("\n🔍 Section 4: Observability Patterns");
    println!("════════════════════════════════════\n");

    describe_observability_patterns();

    // ============================================================
    // SECTION 5: BEST PRACTICES
    // ============================================================
    println!("\n✅ Section 5: Best Practices");
    println!("═════════════════════════════\n");

    println!("1. Create and use a session:");
    println!("   let session = client.create_session().await?;");
    println!("   // Use the session for queries\n");

    println!("2. Monitor lifecycle events:");
    println!("   let session = AgentSession::new_with_lifecycle(config, |event| {{");
    println!("       eprintln!(\"[{{}}::{{}}\", event.description());");
    println!("   }}).await?;\n");

    println!("3. Handle context warnings:");
    println!("   if let SessionEvent::ContextUsageIncreased {{ .. }} = event {{");
    println!("       eprintln!(\"⚠️  Context usage high, consider pruning\");");
    println!("   }}\n");

    println!("4. Log reconnections:");
    println!("   if let SessionEvent::Reconnecting {{ attempt, .. }} = event {{");
    println!("       eprintln!(\"Reconnect attempt {{}}\", attempt);");
    println!("   }}\n");

    println!("═════════════════════════════════════════════════════");
    println!("✨ Session Lifecycle Example Complete!");
    println!("═════════════════════════════════════════════════════\n");

    Ok(())
}

fn demonstrate_lifecycle_events() {
    println!("Lifecycle events provide visibility into session state changes.\n");

    println!("Event Timeline:");
    println!("  1. Created ─────────────→ Session starts");
    println!("  2. ContextUsageIncreased ─→ Memory usage grows");
    println!("  3. ContextPruned ────────→ Memory optimized");
    println!("  4. Reconnecting/Reconnected → Network recovery");
    println!("  5. Closing/Closed ───────→ Session cleanup\n");

    println!("Each event carries:");
    println!("  • session_id: Unique session identifier");
    println!("  • timestamp: When the event occurred (implicit in logging)");
    println!("  • context: Type-specific details (tokens, messages, etc.)\n");
}

async fn demonstrate_raii_cleanup() -> anyhow::Result<()> {
    println!("Session Lifecycle in Action:\n");

    let config = ClaudeAgentClient::builder()
        .api_key(std::env::var("CLAUDE_API_KEY").unwrap())
        .model("claude-3-5-sonnet-20241022")
        .build()?;

    let client = ClaudeAgentClient::new(config);

    println!("Creating session...");
    {
        let _session = client.create_session().await?;

        println!("✅ Session created and ready");
        println!("   (session in use)\n");

        // Session is used here
        println!("   Performing operations...\n");

        // When block exits, session is dropped
        println!("📍 Exiting scope...");
    }
    println!("✅ Session cleaned up on drop\n");

    Ok(())
}

fn describe_event_types() {
    let event_descriptions = vec![
        (
            "Created",
            "Session initialized and ready for use",
            "Logging, monitoring startup",
        ),
        (
            "Forked",
            "New session created from existing session",
            "Tracking session hierarchy",
        ),
        (
            "Closing",
            "Session cleanup in progress",
            "Finishing pending operations",
        ),
        (
            "Closed",
            "Session cleanup complete",
            "Resource deallocation confirmed",
        ),
        (
            "Reconnecting",
            "Attempting to restore broken connection",
            "Network resilience, alerting",
        ),
        (
            "Reconnected",
            "Connection successfully restored",
            "Resuming operations",
        ),
        (
            "Error",
            "An error occurred in the session",
            "Error logging and recovery",
        ),
        (
            "ContextUsageIncreased",
            "Token usage approaching target limit",
            "Proactive memory management",
        ),
        (
            "ContextPruned",
            "Old context removed to manage tokens",
            "Confirming successful optimization",
        ),
    ];

    for (event, meaning, use_case) in event_descriptions {
        println!(
            "  • {} ({} %)",
            event,
            (event.len() as f64 * 100.0 / "Reconnecting".len() as f64) as i32
        );
        println!("    Meaning: {}", meaning);
        println!("    Use case: {}\n", use_case);
    }
}

fn describe_observability_patterns() {
    println!("Pattern 1: Structured Event Logging");
    println!("  fn log_event(event: &SessionEvent) {{");
    println!("      eprintln!(");
    println!("          \"[{{}}:{{}}] {{}}\",");
    println!("          event.session_id(),");
    println!("          event.description()");
    println!("      );");
    println!("  }}\n");

    println!("Pattern 2: Metrics Collection");
    println!("  match event {{");
    println!(
        "      SessionEvent::ContextUsageIncreased {{ tokens_used, target_tokens, .. }} => {{"
    );
    println!("          let util = (*tokens_used as f64 / *target_tokens as f64) * 100.0;");
    println!("          metrics.record_context_usage(util);");
    println!("      }}");
    println!("      _ => {{}}");
    println!("  }}\n");

    println!("Pattern 3: Alerting on Errors");
    println!("  if let SessionEvent::Error {{ session_id, error }} = event {{");
    println!("      alert_team(");
    println!("          format!(\"Session {{}} error: {{}}\", session_id, error)");
    println!("      );");
    println!("  }}\n");
}
