//! Mentor Agent strategy – guides struggling students.

use types::AgentStrategy;

pub struct MentorStrategy {
    pub intervention_threshold: f64,
    pub window_size: usize,
}

impl Default for MentorStrategy {
    fn default() -> Self { Self { intervention_threshold: 0.4, window_size: 5 } }
}

impl AgentStrategy for MentorStrategy {
    fn name(&self) -> &str { "Mentor Agent" }

    fn adapt_difficulty(&self, recent_scores: &[f64]) -> i32 {
        // Mentors always reduce difficulty to unblock the student.
        let window = &recent_scores[recent_scores.len().saturating_sub(self.window_size)..];
        if window.is_empty() { return 2; }
        let avg = window.iter().copied().sum::<f64>() / window.len() as f64;
        if avg < 0.3 { 1 } else { 2 }
    }

    fn needs_mentor(&self, recent_scores: &[f64]) -> bool {
        let window = &recent_scores[recent_scores.len().saturating_sub(self.window_size)..];
        let avg: f64 = window.iter().copied().sum::<f64>() / window.len() as f64;
        window.len() >= 2 && avg < self.intervention_threshold
    }

    fn build_quest_prompt(&self, goal: &str, _biome: &str, _difficulty: i32) -> String {
        format!(
            "Create a bridging teaching quest for a student struggling with \"{goal}\".\n\
             Keep it simple and confidence-building.\n\
             Return JSON: {{\"title\":\"...\",\"description\":\"...\"}}"
        )
    }

    fn build_evaluation_prompt(&self, code: &str, quest_title: &str, _language: &str) -> String {
        format!(
            "A struggling student submitted code for \"{quest_title}\":\n```\n{code}\n```\n\
             Be encouraging. Return JSON: {{\"score\":0.0,\"feedback\":\"supportive sentence\"}}"
        )
    }

    fn build_hint_prompt(&self, concept: &str, student_level: i32) -> String {
        format!(
            "Level-{student_level} student is stuck on \"{concept}\".\n\
             Give a warm, specific one-sentence hint. Don't reveal the answer."
        )
    }
}
