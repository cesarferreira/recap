use chrono::{DateTime, Local};

#[derive(Debug)]
pub struct ContributorStats {
    pub name: String,
    pub commit_count: u32,
    pub last_commit: DateTime<Local>,
    pub first_commit: DateTime<Local>,
}

impl ContributorStats {
    pub fn new(name: String, timestamp: DateTime<Local>) -> Self {
        ContributorStats {
            name,
            commit_count: 1,
            last_commit: timestamp,
            first_commit: timestamp,
        }
    }

    pub fn update(&mut self, timestamp: DateTime<Local>) {
        self.commit_count += 1;
        if timestamp > self.last_commit {
            self.last_commit = timestamp;
        }
        if timestamp < self.first_commit {
            self.first_commit = timestamp;
        }
    }

    pub fn contribution_duration(&self) -> String {
        let duration = self.last_commit.signed_duration_since(self.first_commit);
        let years = duration.num_days() / 365;
        let months = (duration.num_days() % 365) / 30;

        if years > 0 {
            if months > 0 {
                format!("{} years, {} months", years, months)
            } else {
                format!("{} years", years)
            }
        } else if months > 0 {
            format!("{} months", months)
        } else {
            format!("{} days", duration.num_days())
        }
    }

    pub fn format_last_touched(&self) -> String {
        let now = Local::now();
        let duration = now.signed_duration_since(self.last_commit);
        
        if duration.num_days() == 0 {
            "today".to_string()
        } else if duration.num_days() < 7 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_days() < 30 {
            format!("{} weeks ago", duration.num_days() / 7)
        } else if duration.num_days() < 365 {
            format!("{} months ago", duration.num_days() / 30)
        } else {
            format!("{} years ago", duration.num_days() / 365)
        }
    }
} 