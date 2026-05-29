use sqlx::PgPool;
use types::{Quest, QuestStatus, StudentQuest};
use uuid::Uuid;
use crate::error::{DbResult, not_found, unique_err};

pub async fn list(pool: &PgPool) -> DbResult<Vec<Quest>> {
    Ok(sqlx::query_as::<_, Quest>("SELECT * FROM quests ORDER BY difficulty,title").fetch_all(pool).await?)
}

pub async fn list_by_biome(pool: &PgPool, biome_id: Uuid) -> DbResult<Vec<Quest>> {
    Ok(sqlx::query_as::<_, Quest>("SELECT * FROM quests WHERE biome_id=$1 ORDER BY difficulty")
        .bind(biome_id).fetch_all(pool).await?)
}

pub async fn get(pool: &PgPool, id: Uuid) -> DbResult<Quest> {
    not_found(
        sqlx::query_as::<_, Quest>("SELECT * FROM quests WHERE id=$1").bind(id).fetch_one(pool).await,
        "Quest", &id.to_string(),
    )
}

pub async fn create(pool: &PgPool, q: &Quest) -> DbResult<Quest> {
    Ok(sqlx::query_as::<_, Quest>(
        "INSERT INTO quests(id,title,description,quest_type,difficulty,xp_reward,biome_id,requirements,test_cases,status) \
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) RETURNING *"
    ).bind(q.id).bind(&q.title).bind(&q.description).bind(&q.quest_type).bind(q.difficulty)
     .bind(q.xp_reward).bind(q.biome_id).bind(&q.requirements).bind(&q.test_cases).bind(&q.status)
     .fetch_one(pool).await?)
}

pub async fn start_for_student(pool: &PgPool, sid: Uuid, qid: Uuid) -> DbResult<StudentQuest> {
    unique_err(
        sqlx::query_as::<_, StudentQuest>(
            "INSERT INTO student_quests(student_id,quest_id,status) VALUES($1,$2,'in_progress') \
             ON CONFLICT(student_id,quest_id) DO UPDATE SET status='in_progress',attempts=student_quests.attempts+1 \
             RETURNING *"
        ).bind(sid).bind(qid).fetch_one(pool).await,
        "Quest already in terminal state for this student",
    )
}

pub async fn complete_for_student(pool: &PgPool, sid: Uuid, qid: Uuid, passed: bool) -> DbResult<StudentQuest> {
    let status = if passed { QuestStatus::Completed } else { QuestStatus::Failed };
    not_found(
        sqlx::query_as::<_, StudentQuest>(
            "UPDATE student_quests SET status=$3,completed_at=NOW() WHERE student_id=$1 AND quest_id=$2 RETURNING *"
        ).bind(sid).bind(qid).bind(status).fetch_one(pool).await,
        "StudentQuest", &format!("{sid}/{qid}"),
    )
}

pub async fn list_for_student(pool: &PgPool, sid: Uuid) -> DbResult<Vec<StudentQuest>> {
    Ok(sqlx::query_as::<_, StudentQuest>(
        "SELECT * FROM student_quests WHERE student_id=$1 ORDER BY started_at DESC"
    ).bind(sid).fetch_all(pool).await?)
}

pub async fn count_completed(pool: &PgPool, sid: Uuid) -> DbResult<i64> {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM student_quests WHERE student_id=$1 AND status='completed'"
    ).bind(sid).fetch_one(pool).await?;
    Ok(n)
}

pub async fn recent_scores(pool: &PgPool, sid: Uuid, limit: i64) -> DbResult<Vec<f64>> {
    let rows: Vec<(f64,)> = sqlx::query_as(
        "SELECT a.score FROM assessments a \
         JOIN student_quests sq ON sq.quest_id=a.quest_id AND sq.student_id=a.student_id \
         WHERE a.student_id=$1 ORDER BY a.assessed_at DESC LIMIT $2"
    ).bind(sid).bind(limit).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(s,)| s).collect())
}
