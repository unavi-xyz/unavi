use bevy::ecs::component::Component;
use unavi_quota::StockGuard;

#[cfg(not(target_family = "wasm"))] pub mod limiter;

#[derive(Component, Default)]
pub struct QuotaGuards(pub Vec<StockGuard>);

/// Marks a document whose scripts bypass quota enforcement, for trusted system
/// scripts that must keep running even when shared budgets are exhausted.
#[derive(Component)]
pub struct QuotaExempt;
