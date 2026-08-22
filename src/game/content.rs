//! Phase 1 content helpers — reserved for future data-driven progress events.
//! Intentionally minimal for now; keeps the module split prescribed by the spec.

use super::CardType;

/// Stable textual IDs for save compatibility (never persist raw enum ordinals).
pub fn card_stable_id(card: CardType) -> &'static str {
    card.stable_id()
}
