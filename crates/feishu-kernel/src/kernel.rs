//! Microkernel -- manages capability lifecycle and inter-capability events.
//!
//! The kernel is the runtime coordinator. It:
//! 1. Registers capability modules (auth, IM, event, bot, ...)
//! 2. Starts/stops them in order
//! 3. Provides an EventBus for decoupled inter-capability communication
//! 4. Aggregates health-check results

use std::sync::Arc;

use feishu_domain::FeishuError;

use crate::capability::Capability;
use crate::event_bus::EventBus;

/// Microkernel runtime that manages capability lifecycle and events.
pub struct FeishuKernel {
    capabilities: Vec<Arc<dyn Capability>>,
    event_bus: EventBus,
}

impl FeishuKernel {
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            capabilities: Vec::new(),
            event_bus,
        }
    }

    /// Access the shared event bus.
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Register a capability module with the kernel.
    pub fn register(&mut self, cap: Arc<dyn Capability>) {
        tracing::debug!("Kernel: registered capability '{}'", cap.name());
        self.capabilities.push(cap);
    }

    /// Start all registered capabilities in registration order.
    pub async fn start_all(&self) -> Result<(), FeishuError> {
        for cap in &self.capabilities {
            tracing::info!("Kernel: starting '{}'", cap.name());
            cap.start().await.map_err(|e| {
                FeishuError::CapabilityStartFailed(format!("{}: {e}", cap.name()))
            })?;
        }
        tracing::info!(
            "Kernel: all {} capabilities started",
            self.capabilities.len()
        );
        Ok(())
    }

    /// Stop all registered capabilities in reverse order.
    pub async fn stop_all(&self) -> Result<(), FeishuError> {
        for cap in self.capabilities.iter().rev() {
            tracing::info!("Kernel: stopping '{}'", cap.name());
            if let Err(e) = cap.stop().await {
                tracing::error!("Kernel: failed to stop '{}': {e}", cap.name());
            }
        }
        Ok(())
    }

    /// Run health checks on all capabilities, returning (name, healthy) pairs.
    pub async fn health_check_all(&self) -> Vec<(String, bool)> {
        let mut results = Vec::with_capacity(self.capabilities.len());
        for cap in &self.capabilities {
            let healthy = cap.health_check().await;
            results.push((cap.name().to_string(), healthy));
        }
        results
    }

    /// Number of registered capabilities.
    pub fn capability_count(&self) -> usize {
        self.capabilities.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockCap {
        name: &'static str,
    }

    #[async_trait]
    impl Capability for MockCap {
        fn name(&self) -> &str {
            self.name
        }
        async fn health_check(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn kernel_register_and_start() {
        let bus = EventBus::default();
        let mut kernel = FeishuKernel::new(bus);
        kernel.register(Arc::new(MockCap { name: "auth" }));
        kernel.register(Arc::new(MockCap { name: "im" }));
        assert_eq!(kernel.capability_count(), 2);
        assert!(kernel.start_all().await.is_ok());
    }

    #[tokio::test]
    async fn kernel_health_check_all() {
        let bus = EventBus::default();
        let mut kernel = FeishuKernel::new(bus);
        kernel.register(Arc::new(MockCap { name: "auth" }));
        let results = kernel.health_check_all().await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ("auth".into(), true));
    }
}
