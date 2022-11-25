use crate::{session::Activity, Result};
use notify_rust::{Notification, NotificationHandle, Urgency};

pub fn notify(activity: Activity) -> Result<NotificationHandle> {
    let (summary, body, urgency) = match activity {
        Activity::Pomodoro(num) => (
            "Pomodoro completed",
            format!("Pomodoro #{} completed", num),
            Urgency::Normal,
        ),
        Activity::ShortBreak => (
            "Short break ended",
            "Prepare for next pomodoro".to_string(),
            Urgency::Critical,
        ),
        Activity::LongBreak => (
            "Long break ended",
            "Prepare for next pomodoro".to_string(),
            Urgency::Critical,
        ),
    };

    Ok(Notification::new()
        .appname("Solanum")
        .summary(summary)
        .body(&body)
        .urgency(urgency)
        .timeout(5000)
        .show()?)
}
