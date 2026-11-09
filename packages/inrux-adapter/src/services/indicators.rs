//! Bounded activity policy shared by native overlay scrollbars; no application-state writes.
use std::time::{Duration, Instant};
/// Overlay indicators remain visible briefly after movement, then stop requesting native work.
#[derive(Clone, Debug, Default)]
pub(crate) struct Indicators {
    until: Option<Instant>,
    scheduled: bool,
}
impl Indicators {
    /// Extend the activity window without allocating one timer per wheel event.
    pub(crate) fn touch(&mut self) {
        self.until = Some(Instant::now() + Duration::from_millis(700));
    }
    /// Remaining time is absent at rest; native hosts can schedule their own expiry mechanism.
    pub(crate) fn remaining(&self) -> Option<Duration> {
        self.until
            .and_then(|until| until.checked_duration_since(Instant::now()))
            .filter(|delay| !delay.is_zero())
    }
    /// Coalesce asynchronous hosts onto at most one outstanding expiry wake-up per viewport.
    pub(crate) fn schedule(&mut self) -> Option<Duration> {
        let delay = self.remaining()?;
        if self.scheduled {
            return None;
        }
        self.scheduled = true;
        Some(delay)
    }
    /// A completed wake-up allows a newer activity deadline to be scheduled on the next paint.
    pub(crate) fn finished(&mut self) {
        self.scheduled = false;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    /// No sleeps: expired clocks and coalescing are verified directly.
    #[test]
    fn indicators_expire_and_coalesce_without_an_idle_loop() {
        let mut indicators = Indicators::default();
        assert!(indicators.remaining().is_none());
        indicators.touch();
        assert!(indicators.schedule().is_some());
        indicators.touch();
        assert!(indicators.schedule().is_none());
        indicators.finished();
        assert!(indicators.schedule().is_some());
        indicators.until = Some(Instant::now() - Duration::from_secs(1));
        indicators.finished();
        assert!(indicators.schedule().is_none());
    }
}
