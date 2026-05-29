use sqlx::PgPool;
use types::{CreateGroupRequest, GroupMember, GroupMemberDetail, GroupProgressResponse, StudyGroup};
use uuid::Uuid;
use crate::error::{DbError, DbResult, not_found, unique_err};

pub async fn create(pool: &PgPool, req: &CreateGroupRequest) -> DbResult<StudyGroup> {
    unique_err(
        sqlx::query_as::<_, StudyGroup>(
            "INSERT INTO study_groups(name,goal,biome_id,max_members) VALUES($1,$2,$3,$4) RETURNING *"
        ).bind(&req.name).bind(&req.goal).bind(req.biome_id).bind(req.max_members.unwrap_or(10))
         .fetch_one(pool).await,
        "A group with that name already exists",
    )
}

pub async fn get(pool: &PgPool, id: Uuid) -> DbResult<StudyGroup> {
    not_found(
        sqlx::query_as::<_, StudyGroup>("SELECT * FROM study_groups WHERE id=$1")
            .bind(id).fetch_one(pool).await,
        "StudyGroup", &id.to_string(),
    )
}

pub async fn list(pool: &PgPool) -> DbResult<Vec<StudyGroup>> {
    Ok(sqlx::query_as::<_, StudyGroup>(
        "SELECT * FROM study_groups WHERE status='active' ORDER BY created_at DESC"
    ).fetch_all(pool).await?)
}

pub async fn join(pool: &PgPool, gid: Uuid, sid: Uuid) -> DbResult<GroupMember> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM group_members WHERE group_id=$1"
    ).bind(gid).fetch_one(pool).await?;
    let (max,): (i32,) = sqlx::query_as(
        "SELECT max_members FROM study_groups WHERE id=$1"
    ).bind(gid).fetch_one(pool).await.map_err(|_| DbError::NotFound { entity: "StudyGroup", id: gid.to_string() })?;
    if count >= max as i64 {
        return Err(DbError::BadRequest("Group is full".into()));
    }
    unique_err(
        sqlx::query_as::<_, GroupMember>(
            "INSERT INTO group_members(group_id,student_id) VALUES($1,$2) RETURNING *"
        ).bind(gid).bind(sid).fetch_one(pool).await,
        "Student is already a member of this group",
    )
}

pub async fn progress(pool: &PgPool, id: Uuid) -> DbResult<GroupProgressResponse> {
    let group = get(pool, id).await?;
    let members: Vec<GroupMemberDetail> =
        sqlx::query_as::<_, (Uuid, String, String, f64)>(
            "SELECT gm.student_id,s.username,gm.role,gm.contribution \
             FROM group_members gm JOIN students s ON s.id=gm.student_id \
             WHERE gm.group_id=$1 ORDER BY gm.contribution DESC"
        ).bind(id).fetch_all(pool).await?
        .into_iter()
        .map(|(sid, uname, role, contrib)| GroupMemberDetail { student_id: sid, username: uname, role, contribution: contrib })
        .collect();
    let structure = (group.progress > 0.5).then(||
        format!("Tower of {} ({:.0}% complete)", group.goal, group.progress * 100.0)
    );
    Ok(GroupProgressResponse { progress_pct: group.progress * 100.0, collaborative_structure: structure, group, members })
}
