use chrono::{DateTime, Utc};

pub fn format_duration_ago(created: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now - created;

    let seconds = duration.num_seconds();
    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();
    let weeks = duration.num_weeks();
    let months = days / 30; // Approximate months
    let years = days / 365; // Approximate years

    if years > 0 {
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    } else if months > 0 {
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else if weeks > 0 {
        format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
    } else if days > 0 {
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if hours > 0 {
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if minutes > 0 {
        format!(
            "{} minute{} ago",
            minutes,
            if minutes == 1 { "" } else { "s" }
        )
    } else {
        format!(
            "{} second{} ago",
            seconds,
            if seconds == 1 { "" } else { "s" }
        )
    }
}
