//! Curriculum Agent strategy – designs personalised learning journeys.

use types::AgentStrategy;

pub struct CurriculumStrategy;

impl AgentStrategy for CurriculumStrategy {
    fn name(&self) -> &str { "Curriculum Agent" }

    fn adapt_difficulty(&self, recent_scores: &[f64]) -> i32 {
        let window = rolling_window(recent_scores);
        if window.is_empty() { return 3; }
        let avg = avg(window);
        match avg {
            a if a >= 0.9 => 7,
            a if a >= 0.7 => 5,
            a if a >= 0.5 => 3,
            _             => 1,
        }.clamp(1, 10)
    }

    fn needs_mentor(&self, recent_scores: &[f64]) -> bool {
        let window = rolling_window(recent_scores);
        window.len() >= 2 && avg(window) < 0.4
    }

    fn build_quest_prompt(&self, goal: &str, biome: &str, difficulty: i32) -> String {
        format!(
            "Design a learning quest for the '{}' knowledge biome (difficulty {}/10).\n\
             Learning goal: \"{}\"\n\n\
             Respond with ONLY valid JSON:\n\
             {{\"title\":\"<max 80 chars>\",\"description\":\"<max 300 chars>\"}}",
            biome, difficulty, goal
        )
    }

    fn build_evaluation_prompt(&self, code: &str, quest_title: &str, language: &str) -> String {
        format!(
            "Evaluate this {language} code for quest: \"{quest_title}\"\n```\n{code}\n```\n\n\
             Respond with ONLY valid JSON:\n\
             {{\"score\":0.0,\"feedback\":\"one sentence\"}}\n\
             Score 0.0–1.0 = correctness × quality × clarity."
        )
    }

    fn build_hint_prompt(&self, concept: &str, student_level: i32) -> String {
        format!(
            "Level-{student_level} student struggling with \"{concept}\".\n\
             Write one Socratic sentence that nudges without giving the answer."
        )
    }
}

// ── Shared helpers (module-private) ──────────────────────────────────────────

fn rolling_window(scores: &[f64]) -> &[f64] {
    &scores[scores.len().saturating_sub(5)..]
}

fn avg(s: &[f64]) -> f64 {
    s.iter().copied().sum::<f64>() / s.len() as f64
}
