use diesel::pg::PgConnection;
use diesel::prelude::*;

use crate::models;
use crate::phase::*;
use std::time::SystemTime;

pub fn get_phases(conn: &PgConnection) -> Result<Phases, failure::Error> {
    use crate::schema::phases::dsl::*;

    let all_phases: Vec<models::Phase> = phases
        .load::<models::Phase>(conn)
        .map_err(|_| PhaseError::DbError)?;

    Ok(Phases::new(all_phases)?)
}

pub fn insert_user(user_email: &str, conn: &PgConnection) -> Result<i32, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let new_user = models::NewUser { email: user_email };
    let result = diesel::insert_into(users)
        .values(&new_user)
        .get_result::<models::User>(conn)?;

    Ok(result.id)
}

pub fn get_schedulers(
    at_secs: i64,
    conn: &PgConnection,
) -> Result<Vec<models::ScheduleWithMemoryAndUser>, diesel::result::Error> {
    use crate::schema::memories;
    use crate::schema::schedules::dsl::*;
    use crate::schema::users;

    let curr_schedules = schedules
        .filter(next_run.is_not_null().and(next_run.le(at_secs)))
        .inner_join(memories::table.inner_join(users::table))
        .load::<models::ScheduleWithMemoryAndUser>(conn)?;

    Ok(curr_schedules)
}

pub fn update_schedule_time(
    id_to_update: i32,
    new_phase: i32,
    new_time: Option<i64>,
    conn: &PgConnection,
) -> Result<models::Schedule, diesel::result::Error> {
    use crate::schema::schedules::dsl::*;

    diesel::update(schedules.filter(id.eq(id_to_update)))
        .set((next_run.eq(new_time), phase_number.eq(new_phase)))
        .get_result::<models::Schedule>(conn)
}

#[derive(Debug, Fail)]
#[fail(display = "fail to get current millis from Unix epoch. todo: fixme")]
struct TimeError;

#[derive(Debug, Fail)]
#[fail(display = "some database error. todo: fixme")]
struct DbError;

pub fn insert_reminder(
    new_user_id: i32,
    new_topic: Option<&str>,
    new_text: &str,
    conn: &PgConnection,
) -> Result<i32, failure::Error> {
    use crate::schema::memories::dsl::*;
    use crate::schema::schedules::dsl::*;

    let result = conn.transaction::<models::Memory, failure::Error, _>(move || {
        let new_memory = models::NewMemory {
            user_id: new_user_id,
            topic: new_topic,
            text: new_text,
        };
        let created_memory = diesel::insert_into(memories)
            .values(&new_memory)
            .get_result::<models::Memory>(conn)
            .map_err(|_| DbError)?;

        let next_run_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| TimeError)?;

        let new_schedule = models::NewSchedule {
            memory_id: created_memory.id,
            phase_number: 1,
            next_run: Some(next_run_time.as_secs() as i64),
        };

        diesel::insert_into(schedules)
            .values(new_schedule)
            .execute(conn)
            .map_err(|_| DbError)?;

        Ok(created_memory)
    })?;

    Ok(result.id)
}

pub fn get_user(
    user_id: i32,
    conn: &PgConnection,
) -> Result<Option<models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let result = users
        .filter(id.eq(user_id))
        .first::<models::User>(conn)
        .optional()?;

    Ok(result)
}
