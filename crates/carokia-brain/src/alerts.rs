use carokia_decision::ThreatLevel;

/// A security alert raised by the guardian system.
#[derive(Debug, Clone)]
pub struct Alert {
    pub timestamp: std::time::SystemTime,
    pub level: ThreatLevel,
    pub message: String,
    pub location: Option<(f64, f64)>,
}

/// Manages a log of security alerts.
pub struct AlertManager {
    alerts: Vec<Alert>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            alerts: Vec::new(),
        }
    }

    /// Raise a new alert and log it.
    pub fn raise(&mut self, level: ThreatLevel, message: &str, location: Option<(f64, f64)>) {
        let alert = Alert {
            timestamp: std::time::SystemTime::now(),
            level,
            message: message.to_string(),
            location,
        };
        tracing::warn!(
            level = %level,
            message = %message,
            "ALERT raised"
        );
        self.alerts.push(alert);
    }

    /// Return the most recent `count` alerts.
    pub fn recent(&self, count: usize) -> &[Alert] {
        let start = self.alerts.len().saturating_sub(count);
        &self.alerts[start..]
    }

    /// Total number of alerts.
    pub fn count(&self) -> usize {
        self.alerts.len()
    }

    /// Return all alerts.
    pub fn all(&self) -> &[Alert] {
        &self.alerts
    }

    /// Clear all alerts.
    pub fn clear(&mut self) {
        self.alerts.clear();
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_manager_new_is_empty() {
        let mgr = AlertManager::new();
        assert_eq!(mgr.count(), 0);
        assert!(mgr.all().is_empty());
    }

    #[test]
    fn alert_manager_raise_and_count() {
        let mut mgr = AlertManager::new();
        mgr.raise(ThreatLevel::Suspicious, "Test alert", None);
        assert_eq!(mgr.count(), 1);

        mgr.raise(ThreatLevel::Confirmed, "Confirmed alert", Some((3.0, 4.0)));
        assert_eq!(mgr.count(), 2);
    }

    #[test]
    fn alert_manager_recent() {
        let mut mgr = AlertManager::new();
        for i in 0..5 {
            mgr.raise(ThreatLevel::Suspicious, &format!("Alert {}", i), None);
        }

        let recent = mgr.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].message, "Alert 2");
        assert_eq!(recent[2].message, "Alert 4");
    }

    #[test]
    fn alert_manager_recent_more_than_available() {
        let mut mgr = AlertManager::new();
        mgr.raise(ThreatLevel::None, "Only one", None);
        let recent = mgr.recent(10);
        assert_eq!(recent.len(), 1);
    }

    #[test]
    fn alert_stores_location() {
        let mut mgr = AlertManager::new();
        mgr.raise(ThreatLevel::Confirmed, "Intruder", Some((6.0, 4.0)));
        let alert = &mgr.all()[0];
        assert_eq!(alert.location, Some((6.0, 4.0)));
        assert_eq!(alert.level, ThreatLevel::Confirmed);
    }

    #[test]
    fn alert_manager_clear() {
        let mut mgr = AlertManager::new();
        mgr.raise(ThreatLevel::Suspicious, "Test", None);
        assert_eq!(mgr.count(), 1);
        mgr.clear();
        assert_eq!(mgr.count(), 0);
    }
}
