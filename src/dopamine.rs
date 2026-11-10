use std::fs::create_dir_all;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::storage::get_data_dir;

pub const DEFAULT_SECS_PER_DOPAMINE_POINT: u64 = 300; // 5 minutes = 1 point

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RewardCategory {
    Quick,  // 5–10 min (kaffe, strekk, favorittsang)
    Break,  // 15–20 min (tur, lunsj, youtube med god samvittighet)
    Deep,   // 30+ min (spill en runde, se en episode, hobby)
    Custom, // egne tilpassede belønninger
}

impl RewardCategory {
    pub fn display_name(&self, is_norwegian: bool) -> &'static str {
        match (self, is_norwegian) {
            (RewardCategory::Quick, true) => "Kjapp belønning (5–10 min)",
            (RewardCategory::Quick, false) => "Quick Reward (5–10 min)",
            (RewardCategory::Break, true) => "Mellomstor pause (15–25 min)",
            (RewardCategory::Break, false) => "Medium Break (15–25 min)",
            (RewardCategory::Deep, true) => "Stor kveldsbelønning (30+ min)",
            (RewardCategory::Deep, false) => "Deep Reward (30+ min)",
            (RewardCategory::Custom, true) => "Egendefinert",
            (RewardCategory::Custom, false) => "Custom",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewardItem {
    pub id: String,
    pub title: String,
    pub cost: u32,
    pub category: RewardCategory,
    pub is_custom: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewardRedemption {
    pub timestamp: u64,
    pub reward_id: String,
    pub reward_title: String,
    pub cost: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DopamineAccount {
    pub balance: u32,
    pub total_earned: u32,
    pub total_spent: u32,
    pub rewards: Vec<RewardItem>,
    pub redemptions: Vec<RewardRedemption>,
    pub accrued_focus_secs: u64,
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl Default for DopamineAccount {
    fn default() -> Self {
        Self::new()
    }
}

impl DopamineAccount {
    pub fn new() -> Self {
        Self {
            balance: 0,
            total_earned: 0,
            total_spent: 0,
            rewards: Self::default_rewards(),
            redemptions: Vec::new(),
            accrued_focus_secs: 0,
        }
    }

    pub fn default_rewards() -> Vec<RewardItem> {
        vec![
            RewardItem {
                id: "coffee".into(),
                title: "Kaffepause eller varm te".into(),
                cost: 5,
                category: RewardCategory::Quick,
                is_custom: false,
            },
            RewardItem {
                id: "stretch".into(),
                title: "Kort strekk og frisk luft".into(),
                cost: 5,
                category: RewardCategory::Quick,
                is_custom: false,
            },
            RewardItem {
                id: "song".into(),
                title: "Favorittsang med full guffe".into(),
                cost: 5,
                category: RewardCategory::Quick,
                is_custom: false,
            },
            RewardItem {
                id: "youtube".into(),
                title: "15 min YouTube / Reddit (guilt-free)".into(),
                cost: 15,
                category: RewardCategory::Break,
                is_custom: false,
            },
            RewardItem {
                id: "walk".into(),
                title: "Gå en tur i nabolaget".into(),
                cost: 20,
                category: RewardCategory::Break,
                is_custom: false,
            },
            RewardItem {
                id: "game".into(),
                title: "Spille en runde eller se en episode".into(),
                cost: 30,
                category: RewardCategory::Deep,
                is_custom: false,
            },
        ]
    }

    /// Add arbitrary points to the balance
    pub fn earn_points(&mut self, amount: u32) -> u32 {
        if amount == 0 {
            return self.balance;
        }
        self.balance = self.balance.saturating_add(amount);
        self.total_earned = self.total_earned.saturating_add(amount);
        self.balance
    }

    /// Record elapsed focus seconds. Every `secs_per_point` yields 1 dopamine point.
    /// Returns the number of new points earned in this update.
    pub fn record_focus_time(&mut self, delta_secs: u64, secs_per_point: u64) -> u32 {
        let step = secs_per_point.max(60);
        self.accrued_focus_secs = self.accrued_focus_secs.saturating_add(delta_secs);
        let earned = (self.accrued_focus_secs / step) as u32;
        if earned > 0 {
            self.accrued_focus_secs %= step;
            self.earn_points(earned);
        }
        earned
    }

    /// Bonus for completing a 5-minute Micro-Start (+1 point)
    pub fn earn_micro_start_bonus(&mut self) -> u32 {
        self.earn_points(1);
        1
    }

    /// Bonus for parking an impulse rather than following a distraction (+1 point)
    pub fn earn_impulse_shield_bonus(&mut self) -> u32 {
        self.earn_points(1);
        1
    }

    pub fn can_redeem(&self, reward_id: &str) -> bool {
        if let Some(r) = self.rewards.iter().find(|r| r.id == reward_id) {
            self.balance >= r.cost
        } else {
            false
        }
    }

    pub fn redeem(&mut self, reward_id: &str) -> Result<RewardRedemption, String> {
        let item = self
            .rewards
            .iter()
            .find(|r| r.id == reward_id)
            .cloned()
            .ok_or_else(|| format!("Belønning med ID '{}' finnes ikke", reward_id))?;

        if self.balance < item.cost {
            return Err(format!(
                "Ikke nok poeng for '{}' (krever {}, du har {})",
                item.title, item.cost, self.balance
            ));
        }

        self.balance = self.balance.saturating_sub(item.cost);
        self.total_spent = self.total_spent.saturating_add(item.cost);

        let redemption = RewardRedemption {
            timestamp: now_epoch(),
            reward_id: item.id.clone(),
            reward_title: item.title.clone(),
            cost: item.cost,
        };

        self.redemptions.push(redemption.clone());
        Ok(redemption)
    }

    pub fn add_custom_reward(&mut self, title: String, cost: u32, category: RewardCategory) -> RewardItem {
        let clean_title = title.trim().to_string();
        let cost_clamped = cost.max(1);
        let id = format!(
            "custom_{}_{}",
            now_epoch(),
            self.rewards.len() + 1
        );

        let item = RewardItem {
            id,
            title: clean_title,
            cost: cost_clamped,
            category,
            is_custom: true,
        };

        self.rewards.push(item.clone());
        item
    }

    pub fn remove_custom_reward(&mut self, id: &str) -> bool {
        if let Some(pos) = self.rewards.iter().position(|r| r.id == id && r.is_custom) {
            self.rewards.remove(pos);
            true
        } else {
            false
        }
    }
}

pub fn get_dopamine_file() -> PathBuf {
    get_data_dir().join("dopamine.json")
}

pub fn load_dopamine_account() -> DopamineAccount {
    let path = get_dopamine_file();
    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(account) = serde_json::from_slice::<DopamineAccount>(&bytes) {
            return account;
        }
    }
    DopamineAccount::default()
}

pub fn save_dopamine_account(account: &DopamineAccount) -> Result<(), std::io::Error> {
    let path = get_dopamine_file();
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(account)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_account_creation() {
        let acc = DopamineAccount::new();
        assert_eq!(acc.balance, 0);
        assert_eq!(acc.total_earned, 0);
        assert_eq!(acc.total_spent, 0);
        assert_eq!(acc.accrued_focus_secs, 0);
        assert!(acc.rewards.len() >= 6);
    }

    #[test]
    fn test_earn_points_and_focus_accrual() {
        let mut acc = DopamineAccount::new();
        acc.earn_points(5);
        assert_eq!(acc.balance, 5);
        assert_eq!(acc.total_earned, 5);

        // 300s = 5m -> 1 point
        let earned = acc.record_focus_time(300, 300);
        assert_eq!(earned, 1);
        assert_eq!(acc.balance, 6);
        assert_eq!(acc.accrued_focus_secs, 0);

        // Sub-interval accumulation: 150s + 150s = 300s -> 1 point
        let e1 = acc.record_focus_time(150, 300);
        assert_eq!(e1, 0);
        assert_eq!(acc.accrued_focus_secs, 150);

        let e2 = acc.record_focus_time(150, 300);
        assert_eq!(e2, 1);
        assert_eq!(acc.balance, 7);
        assert_eq!(acc.accrued_focus_secs, 0);
    }

    #[test]
    fn test_bonuses() {
        let mut acc = DopamineAccount::new();
        acc.earn_micro_start_bonus();
        acc.earn_impulse_shield_bonus();
        assert_eq!(acc.balance, 2);
        assert_eq!(acc.total_earned, 2);
    }

    #[test]
    fn test_redeem_reward() {
        let mut acc = DopamineAccount::new();
        acc.earn_points(20);

        assert!(acc.can_redeem("coffee"));
        assert!(acc.can_redeem("youtube"));
        assert!(!acc.can_redeem("game")); // costs 30

        let res = acc.redeem("coffee");
        assert!(res.is_ok());
        assert_eq!(acc.balance, 15);
        assert_eq!(acc.total_spent, 5);
        assert_eq!(acc.redemptions.len(), 1);

        let res_fail = acc.redeem("game");
        assert!(res_fail.is_err());
        assert_eq!(acc.balance, 15);
    }

    #[test]
    fn test_custom_rewards() {
        let mut acc = DopamineAccount::new();
        let item = acc.add_custom_reward("Spille gitar".into(), 10, RewardCategory::Break);
        assert_eq!(item.title, "Spille gitar");
        assert_eq!(item.cost, 10);
        assert!(item.is_custom);

        assert!(acc.rewards.iter().any(|r| r.id == item.id));

        let removed = acc.remove_custom_reward(&item.id);
        assert!(removed);
        assert!(!acc.rewards.iter().any(|r| r.id == item.id));

        // Default reward cannot be removed via remove_custom_reward
        let def_removed = acc.remove_custom_reward("coffee");
        assert!(!def_removed);
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut acc = DopamineAccount::new();
        acc.earn_points(25);
        let _ = acc.redeem("coffee");
        acc.add_custom_reward("Kveldstur".into(), 15, RewardCategory::Break);

        let json = serde_json::to_string_pretty(&acc).expect("serialize");
        let decoded: DopamineAccount = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(decoded.balance, acc.balance);
        assert_eq!(decoded.total_earned, acc.total_earned);
        assert_eq!(decoded.total_spent, acc.total_spent);
        assert_eq!(decoded.rewards.len(), acc.rewards.len());
        assert_eq!(decoded.redemptions.len(), acc.redemptions.len());
    }
}
