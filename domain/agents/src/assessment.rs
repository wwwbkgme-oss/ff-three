//! Assessment Agent strategy – evaluates student submissions.

use types::AgentStrategy;

pub struct AssessmentStrategy {
    pub pass_threshold: f64,
}

impl Default for AssessmentStrategy {
    fn default() -> Self { Self { pass_threshold: 0.7 } }
}

impl AgentStrategy for AssessmentStrategy {
    fn name(&self) -> &str { "Assessment Agent" }

    fn adapt_difficulty(&self, recent_scores: &[f64]) -> i32 {
        let window = &recent_scores[recent_scores.len().saturating_sub(5)..];
        if window.is_empty() { return 5; }
        let avg = window.iter().copied().sum::<f64>() / window.len() as f64;
        if avg >= 0.85 { 8 } else if avg >= 0.7 { 6 } else { 4 }
    }

    fn needs_mentor(&self, recent_scores: &[f64]) -> bool {
        recent_scores.len() >= 3
            && recent_scores.iter().rev().take(3).all(|&s| s < self.pass_threshold)
    }

    fn build_quest_prompt(&self, goal: &str, biome: &str, difficulty: i32) -> String {
        format!(
            "Create an assessment quest for '{}' (difficulty {}/10) in the '{}' biome.\n\
             Return JSON: {{\"title\":\"...\",\"description\":\"...\"}}",
            goal, difficulty, biome
        )
    }

    fn build_evaluation_prompt(&self, code: &str, quest_title: &str, language: &str) -> String {
        format!(
            "Rigorously assess this {language} submission for \"{quest_title}\":\n```\n{code}\n```\n\
             Return JSON: {{\"score\":0.0,\"feedback\":\"sentence\"}}"
        )
    }

    fn build_hint_prompt(&self, concept: &str, student_level: i32) -> String {
        format!(
            "Level-{student_level} student needs assessment guidance on \"{concept}\".\n\
             Write a targeted one-sentence hint."
        )
    }
}
