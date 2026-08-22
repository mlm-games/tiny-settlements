use bevy::prelude::*;

use super::{Card, GameEvent, PendingDespawns, PendingGameEvents};

pub const EXCHANGE_MIN: Vec2 = Vec2::new(330.0, -275.0);
pub const EXCHANGE_MAX: Vec2 = Vec2::new(465.0, -125.0);

#[derive(Resource, Default)]
pub struct RunEconomy {
    pub dew: u32,
    pub total_earned: u32,
    pub total_spent: u32,
}

impl RunEconomy {
    pub fn can_afford(&self, amount: u32) -> bool {
        self.dew >= amount
    }

    pub fn earn(&mut self, amount: u32) {
        self.dew = self.dew.saturating_add(amount);
        self.total_earned = self.total_earned.saturating_add(amount);
    }

    pub fn spend(&mut self, amount: u32) -> bool {
        if self.dew < amount {
            return false;
        }
        self.dew -= amount;
        self.total_spent = self.total_spent.saturating_add(amount);
        true
    }
}

pub fn point_in_exchange(point: Vec2) -> bool {
    point.x >= EXCHANGE_MIN.x
        && point.x <= EXCHANGE_MAX.x
        && point.y >= EXCHANGE_MAX.y.min(EXCHANGE_MIN.y)
        && point.y <= EXCHANGE_MAX.y.max(EXCHANGE_MIN.y)
        // explicit y check (avoid min/max confusion)
        && point.y >= EXCHANGE_MIN.y
        && point.y <= EXCHANGE_MAX.y
}

pub fn try_sell_card(
    entity: Entity,
    position: Vec2,
    card: &Card,
    economy: &mut RunEconomy,
    despawns: &mut PendingDespawns,
    events: &mut PendingGameEvents,
) -> bool {
    if !point_in_exchange(position) {
        return false;
    }
    let Some(value) = card.card_type.sell_value() else {
        return false;
    };
    economy.earn(value);
    despawns.0.push(entity);
    events.0.push(GameEvent::Sold {
        card: card.card_type,
        value,
    });
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn economy_earn_spend() {
        let mut e = RunEconomy::default();
        e.earn(10);
        assert_eq!(e.dew, 10);
        assert_eq!(e.total_earned, 10);
        assert!(e.can_afford(4));
        assert!(e.spend(4));
        assert_eq!(e.dew, 6);
        assert_eq!(e.total_spent, 4);
        assert!(!e.spend(100));
        assert_eq!(e.dew, 6);
    }

    #[test]
    fn point_in_exchange_bounds() {
        assert!(point_in_exchange(Vec2::new(400.0, -200.0)));
        assert!(point_in_exchange(EXCHANGE_MIN));
        assert!(point_in_exchange(EXCHANGE_MAX));
        assert!(!point_in_exchange(Vec2::new(0.0, 0.0)));
        assert!(!point_in_exchange(Vec2::new(329.0, -200.0)));
    }
}
