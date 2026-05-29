//! Agent Orchestrator – selects the correct strategy for each context.
//!
//! Pure logic: picks the right `AgentStrategy`, computes prompts, adapts
//! difficulty.  No I/O happens here.

use std::sync::Arc;

use types::AgentStrategy;

use super::{
    assessment::AssessmentStrategy,
    curriculum::CurriculumStrategy,
    mentor::MentorStrategy,
};

pub struct Orchestrator {
    curriculum: Arc<CurriculumStrategy>,
    assessment: Arc<AssessmentStrategy>,
    mentor:     Arc<MentorStrategy>,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self {
            curriculum: Arc::new(CurriculumStrategy),
            assessment: Arc::new(AssessmentStrategy::default()),
            mentor:     Arc::new(MentorStrategy::default()),
        }
    }
}

impl Orchestrator {
    pub fn new() -> Self { Self::default() }

    // ── Strategy selection ────────────────────────────────────────────────────

    /// Return the active strategy given the student's recent performance.
    /// Mentor takes over when the student is consistently struggling.
    pub fn select(&self, recent_scores: &[f64]) -> &dyn AgentStrategy {
        if self.mentor.needs_mentor(recent_scores) {
            self.mentor.as_ref()
        } else {
            self.curriculum.as_ref()
        }
    }

    /// Return the assessment strategy (independent of performance).
    pub fn assessor(&self) -> &dyn AgentStrategy {
        self.assessment.as_ref()
    }

    // ── Delegation helpers ────────────────────────────────────────────────────

    pub fn adapt_difficulty(&self, recent_scores: &[f64]) -> i32 {
        self.select(recent_scores).adapt_difficulty(recent_scores)
    }

    pub fn needs_mentor(&self, recent_scores: &[f64]) -> bool {
        self.mentor.needs_mentor(recent_scores)
    }

    pub fn build_quest_prompt(&self, goal: &str, biome: &str, difficulty: i32, recent_scores: &[f64]) -> String {
        self.select(recent_scores).build_quest_prompt(goal, biome, difficulty)
    }

    pub fn build_evaluation_prompt(&self, code: &str, quest_title: &str, language: &str) -> String {
        self.assessor().build_evaluation_prompt(code, quest_title, language)
    }

    pub fn build_hint_prompt(&self, concept: &str, student_level: i32, recent_scores: &[f64]) -> String {
        self.select(recent_scores).build_hint_prompt(concept, student_level)
    }
}
