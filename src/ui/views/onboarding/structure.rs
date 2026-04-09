use crate::i18n::t;

#[derive(Debug, Clone)]
pub enum OnboardingMessage {
    Next,
    Previous,
    Skip,
    Complete,
}

pub enum OnboardingEvent {
    Finished,
    Skipped,
}

pub struct OnboardingView {
    pub current_step: u8,
    pub total_steps: u8,
}
