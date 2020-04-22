use crate::schema::memories;
use crate::schema::schedules;
use crate::schema::users;

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: &'a str,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct Memory {
    pub id: i32,
    pub user_id: i32,
    pub topic: Option<String>,
    pub text: String,
}

#[derive(Insertable)]
#[table_name = "memories"]
pub struct NewMemory<'a> {
    pub user_id: i32,
    pub topic: Option<&'a str>,
    pub text: &'a str,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct Phase {
    pub id: i32,
    pub number: i32,
    pub seconds_to_wait: i64,
}

#[derive(Insertable)]
#[table_name = "schedules"]
pub struct NewSchedule {
    pub memory_id: i32,
    pub phase_number: i32,
    pub next_run: Option<i64>,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct Schedule {
    pub id: i32,
    pub memory_id: i32,
    pub phase_number: i32,
    pub next_run: Option<i64>,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct MemoryWithUser {
    pub memory: Memory,
    pub user: User,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct ScheduleWithMemoryAndUser {
    pub schedule: Schedule,
    pub memory_with_user: MemoryWithUser,
}
