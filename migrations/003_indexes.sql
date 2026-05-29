-- 003: Indexes
CREATE INDEX idx_students_rank        ON students      (level DESC, xp DESC);
CREATE INDEX idx_students_biome       ON students      (current_biome_id) WHERE current_biome_id IS NOT NULL;
CREATE INDEX idx_quests_biome         ON quests        (biome_id);
CREATE INDEX idx_quests_difficulty    ON quests        (difficulty);
CREATE INDEX idx_sq_student           ON student_quests(student_id);
CREATE INDEX idx_sq_status            ON student_quests(status);
CREATE INDEX idx_assess_student       ON assessments   (student_id);
CREATE INDEX idx_assess_quest         ON assessments   (quest_id);
CREATE INDEX idx_sandbox_student      ON sandbox_runs  (student_id);
CREATE INDEX idx_sandbox_status       ON sandbox_runs  (status);
CREATE INDEX idx_gm_student           ON group_members (student_id);
CREATE INDEX idx_achieve_student      ON achievements  (student_id);
CREATE INDEX idx_cert_student         ON certifications(student_id);
