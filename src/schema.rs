table! {
    memories (id) {
        id -> Int4,
        user_id -> Int4,
        topic -> Nullable<Varchar>,
        text -> Text,
    }
}

table! {
    phases (id) {
        id -> Int4,
        phase_number -> Int4,
        seconds_to_wait -> Int8,
    }
}

table! {
    schedules (id) {
        id -> Int4,
        memory_id -> Int4,
        phase_number -> Int4,
        next_run -> Nullable<Int8>,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
    }
}

joinable!(memories -> users (user_id));
joinable!(schedules -> memories (memory_id));

allow_tables_to_appear_in_same_query!(
    memories,
    phases,
    schedules,
    users,
);
