use crate::db_actions;
use crate::phase::Phases;
use crate::DbPool;
use diesel::pg::PgConnection;
use log::{debug, error, info};
use std::time::Duration;
use std::time::SystemTime;

#[derive(Debug, Fail)]
#[fail(display = "fail to check schedulers. todo: fixme")]
struct RunError;

pub fn start_checking_thread(phases: Phases, sleep_duration: Duration, pool: DbPool) {
    let mut sleep_interval = tokio::time::interval(sleep_duration);
    tokio::spawn(async move {
        loop {
            sleep_interval.tick().await;
            let conn = pool.get().expect("couldn't get db connection from pool");
            match one_run(&phases, &conn) {
                Ok(_) => debug!("successfully check all schedulers"),
                Err(err) => error!("{}", err),
            }
        }
    });
}

fn one_run(phases: &Phases, conn: &PgConnection) -> Result<(), failure::Error> {
    let curr_seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| RunError)?;
    let schedulers =
        db_actions::get_schedulers(curr_seconds.as_secs() as i64, conn).map_err(|_| RunError)?;

    let schedulers_to_update = schedulers.iter().filter(|sch_with_memory| {
        let email = &sch_with_memory.memory_with_user.user.email;
        let topic = sch_with_memory.memory_with_user.memory.topic.as_deref();
        let text = &sch_with_memory.memory_with_user.memory.text;
        try_to_send_email(email, topic, text).is_ok()
    });

    schedulers_to_update.for_each(|sch_with_memory| {
        info!("scheduler to check: {:?}", sch_with_memory);
        let next_phase_num = sch_with_memory.schedule.phase_number + 1;
        let secs_to_add = phases.get(next_phase_num).map(|ph| ph.seconds_to_wait);

        let update_res = db_actions::update_schedule_time(
            sch_with_memory.schedule.id,
            next_phase_num,
            secs_to_add.map(|s| s + sch_with_memory.schedule.next_run.unwrap()),
            conn,
        );
        match update_res {
            Ok(_) => debug!(
                "successfully update next run time for schedule with id '{}', add seconds '{}'",
                sch_with_memory.schedule.id,
                secs_to_add.unwrap_or(-1)
            ),
            Err(err) => error!(
                "fail to updare next run time for schedule with id '{}', reason: '{}'",
                sch_with_memory.schedule.id, err
            ),
        }
    });

    // todo: some logic with schedulers.
    // 1. Send email
    // 2. update phase and time.

    Ok(())
}

use lettre::smtp::authentication::Credentials;
use lettre::SendableEmail;
use lettre::SmtpTransport;
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;

#[derive(Debug, Fail)]
#[fail(display = "fail to send email")]
struct FailToSendEmail;

fn try_to_send_email(
    address: &str,
    topic: Option<&str>,
    text: &str,
) -> Result<lettre::smtp::response::Response, failure::Error> {
    let cred_email: String = "{your_email_here}".to_string();
    let cred_password: String = "{password}".to_string();

    let creds: Credentials = Credentials::new(cred_email.clone(), cred_password);

    let mut mailer: SmtpTransport = SmtpClient::new_simple("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .transport();

    let email: SendableEmail = EmailBuilder::new()
        .to(address)
        .from(cred_email)
        .subject(topic.unwrap_or_default())
        .text(text)
        .build()
        .unwrap()
        .into();

    Ok(mailer.send(email).map_err(|_| FailToSendEmail)?)
}
